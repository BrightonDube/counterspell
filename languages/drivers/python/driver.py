"""Counterspell Python driver, protocol version 1 (see languages/protocol.md).

The driver loads the user's solution once. It then reads one call per stdin line and writes
one answer line per call to stdout. It never prints a verdict, an expected value, a timing or
a complexity class: the trusted judge owns all of those. Diagnostics go to stderr only.

The sandbox runs this as `python3 -I -S driver.py`. `--mode bench` is accepted and uses the
same loop; the judge does all timing.
"""

import argparse
import contextlib
import io
import json
import math
import sys
import traceback

PROTOCOL_VERSION = 1
SIGNATURE_PATH = "/box/signature.json"
SOLUTION_PATH = "/box/solution.py"
MAX_SAFE_INTEGER = 2**53 - 1
KINDS = frozenset({"int", "float", "bool", "string", "list", "tuple", "null"})
EXIT_PROTOCOL = 2
EXIT_USER = 1


class DriverFailure(Exception):
    """A setup or protocol failure. The driver reports it on stderr and exits non-zero."""

    def __init__(self, message, exit_code=EXIT_PROTOCOL):
        super().__init__(message)
        self.exit_code = exit_code


class Signature:
    """The public function signature: its name, parameter types and return type."""

    def __init__(self, function, parameters, returns):
        self.function = function
        self.parameters = parameters
        self.returns = returns


class _Sink(io.TextIOBase):
    """A stream that drops everything written to it. User prints go here, not to the answers."""

    def write(self, text):
        return len(text)


@contextlib.contextmanager
def quiet():
    """Hides stdout and stdin while user code runs, then restores them."""
    saved_stdout, saved_stdin = sys.stdout, sys.stdin
    sys.stdout, sys.stdin = _Sink(), io.StringIO("")
    try:
        yield
    finally:
        sys.stdout, sys.stdin = saved_stdout, saved_stdin


def diagnose(message):
    """Writes one diagnostic line to stderr."""
    sys.stderr.write("driver: " + message + "\n")
    sys.stderr.flush()


def check_type(descriptor):
    """Checks one type descriptor from the signature and returns it unchanged."""
    if not isinstance(descriptor, dict):
        raise DriverFailure("a type in the signature is not an object")
    kind = descriptor.get("kind")
    if kind not in KINDS:
        raise DriverFailure(f"unknown type kind {kind!r} in the signature")
    if not isinstance(descriptor.get("nullable", False), bool):
        raise DriverFailure("the nullable flag must be true or false")
    if kind == "list":
        check_type(descriptor.get("items"))
    if kind == "tuple":
        items = descriptor.get("items")
        if not isinstance(items, list):
            raise DriverFailure("a tuple type needs a list of item types")
        for item in items:
            check_type(item)
    return descriptor


def load_signature(path):
    """Reads and checks the signature file the judge wrote into the sandbox."""
    try:
        with open(path, "rb") as handle:
            data = json.loads(handle.read().decode("utf-8"))
    except (OSError, ValueError) as error:
        raise DriverFailure(f"cannot read the signature: {error}") from error
    version = data.get("protocol_version") if isinstance(data, dict) else None
    if not is_json_integer(version) or version != PROTOCOL_VERSION:
        raise DriverFailure("the signature has an unsupported protocol version")
    function = data.get("function")
    if not isinstance(function, str) or not function.isidentifier():
        raise DriverFailure("the signature has no valid function name")
    parameters = data.get("parameters")
    if not isinstance(parameters, list):
        raise DriverFailure("the signature has no parameter list")
    types = []
    for parameter in parameters:
        if not isinstance(parameter, dict):
            raise DriverFailure("a parameter in the signature is not an object")
        types.append(check_type(parameter.get("type")))
    return Signature(function, types, check_type(data.get("returns")))


def is_json_integer(value):
    """True for an int inside the exact JSON range. A bool is not an integer here."""
    return isinstance(value, int) and not isinstance(value, bool) and abs(value) <= MAX_SAFE_INTEGER


def is_valid_unicode(text):
    """False for strings with lone surrogates, which cannot be encoded as UTF-8."""
    try:
        text.encode("utf-8")
    except UnicodeEncodeError:
        return False
    return True


def conforms(value, descriptor):
    """True when a JSON-safe value matches a type descriptor. It never converts the value."""
    if value is None:
        return descriptor["kind"] == "null" or descriptor.get("nullable", False)
    kind = descriptor["kind"]
    if kind == "int":
        return is_json_integer(value)
    if kind == "float":
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            return False
        if isinstance(value, int):
            return is_json_integer(value)
        return math.isfinite(value)
    if kind == "bool":
        return isinstance(value, bool)
    if kind == "string":
        return isinstance(value, str) and is_valid_unicode(value)
    if kind == "list":
        return isinstance(value, (list, tuple)) and all(
            conforms(item, descriptor["items"]) for item in value
        )
    if kind == "tuple":
        items = descriptor["items"]
        return (
            isinstance(value, (list, tuple))
            and len(value) == len(items)
            and all(conforms(item, item_type) for item, item_type in zip(value, items))
        )
    return False


def reject_duplicate_keys(pairs):
    """An object hook that refuses a JSON object with a repeated key."""
    keys = [key for key, _ in pairs]
    if len(set(keys)) != len(keys):
        raise DriverFailure("the call repeats a key")
    return dict(pairs)


def refuse_constant(name):
    """Refuses NaN, Infinity and -Infinity, which are not JSON."""
    raise DriverFailure(f"the call contains the non-JSON value {name}")


def parse_call(line, signature):
    """Checks one call line and returns its id and argument list."""
    if line.strip() == b"":
        raise DriverFailure("a blank call line is refused")
    try:
        text = line.decode("utf-8")
    except UnicodeDecodeError as error:
        raise DriverFailure("the call line is not UTF-8") from error
    try:
        call = json.loads(
            text, object_pairs_hook=reject_duplicate_keys, parse_constant=refuse_constant
        )
    except ValueError as error:
        raise DriverFailure("the call line is not one JSON value") from error
    if not isinstance(call, dict) or set(call) != {"t", "args"}:
        raise DriverFailure("a call must have exactly the keys t and args")
    call_id = call["t"]
    if not is_json_integer(call_id) or call_id < 0:
        raise DriverFailure("the call id must be a non-negative integer")
    args = call["args"]
    if not isinstance(args, list) or len(args) != len(signature.parameters):
        raise DriverFailure("the argument count does not match the signature")
    for index, (value, descriptor) in enumerate(zip(args, signature.parameters)):
        if not conforms(value, descriptor):
            raise DriverFailure(f"argument {index} does not match the signature")
    return call_id, args


def load_function(signature):
    """Runs the solution once and returns the function named in the signature."""
    try:
        with open(SOLUTION_PATH, "rb") as handle:
            source = handle.read()
    except OSError as error:
        raise DriverFailure(f"cannot read the solution: {error}") from error
    namespace = {"__name__": "counterspell_solution"}
    try:
        with quiet():
            exec(compile(source, SOLUTION_PATH, "exec"), namespace)  # nosemgrep: python.lang.security.audit.exec-detected.exec-detected
    except BaseException:
        # The driver line goes first: the judge reads the last stderr line as
        # `ExceptionClass: message`, so the traceback tail must stay last.
        diagnose("the solution failed while loading")
        traceback.print_exc()
        sys.exit(EXIT_PROTOCOL)
    function = namespace.get(signature.function)
    if not callable(function):
        raise DriverFailure(f"the solution does not define {signature.function}")
    return function


def call_solution(function, args):
    """Calls the user's function with its prints discarded. A failure ends the driver."""
    try:
        with quiet():
            return function(*args)
    except BaseException:
        # Same ordering: `driver:` diagnostic first, traceback tail last.
        diagnose("the solution raised an exception")
        traceback.print_exc()
        sys.exit(EXIT_USER)


def write_answer(stream, call_id, out):
    """Writes one answer line and flushes it. JSON escapes newlines, so it is one line."""
    line = json.dumps({"t": call_id, "out": out}, allow_nan=False, separators=(",", ":"))
    stream.write((line + "\n").encode("ascii"))
    stream.flush()


def serve(function, signature, requests, answers):
    """Answers calls until the judge closes stdin."""
    for raw in iter(requests.readline, b""):
        call_id, args = parse_call(raw, signature)
        result = call_solution(function, args)
        if not conforms(result, signature.returns):
            raise DriverFailure("the return value is not JSON-safe for the signature")
        write_answer(answers, call_id, result)


def main(argv=None):
    """Entry point. Returns the exit code."""
    parser = argparse.ArgumentParser(prog="driver.py")
    parser.add_argument("--mode", choices=("normal", "bench"), default="normal")
    parser.parse_args(argv)
    try:
        signature = load_signature(SIGNATURE_PATH)
        function = load_function(signature)
        serve(function, signature, sys.stdin.buffer, sys.stdout.buffer)
    except DriverFailure as failure:
        diagnose(str(failure))
        return failure.exit_code
    return 0


if __name__ == "__main__":
    sys.exit(main())
