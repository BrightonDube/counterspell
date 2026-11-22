'use strict';
/**
 * Counterspell Node.js driver, protocol version 1 (see languages/protocol.md).
 *
 * The driver loads the user's CommonJS solution once. It then reads one call per stdin line and
 * writes one answer line per call to stdout. It never prints a verdict, an expected value, a
 * timing or a complexity class: the trusted judge owns all of those. Diagnostics go to stderr.
 *
 * The sandbox runs this as `node --max-old-space-size=192 driver.js`. `--mode bench` is accepted
 * and uses the same loop; the judge does all timing.
 *
 * Known difference from the Python driver: this driver does not reject a call line that repeats
 * a key, because `JSON.parse` keeps the last copy. The judge writes every call line itself, so
 * a repeated key cannot come from the judge.
 */

const fs = require('fs');
const readline = require('readline');

const PROTOCOL_VERSION = 1;
const SIGNATURE_PATH = '/box/signature.json';
const SOLUTION_PATH = '/box/solution.js';
const KINDS = new Set(['int', 'float', 'bool', 'string', 'list', 'tuple', 'null']);
const IDENTIFIER = /^[A-Za-z_$][A-Za-z0-9_$]*$/;
const EXIT_PROTOCOL = 2;
const EXIT_USER = 1;
const MODES = new Set(['normal', 'bench']);
const STDERR_CAP = 64 * 1024;
const STDERR_TRUNCATED = '\n[stderr truncated]\n';
const CALL_STDERR_MARKER = '\x1ecounterspell:stderr\n';
const writeOut = process.stdout.write.bind(process.stdout);
const writeErr = process.stderr.write.bind(process.stderr);
const writeFsSync = fs.writeSync.bind(fs);
const writeFs = fs.write.bind(fs);
const exitProcess = process.exit.bind(process);
let activeCapture = null;

/** A setup or protocol failure. The driver reports it on stderr and exits with `exitCode`. */
class DriverFailure extends Error {
  constructor(message, exitCode = EXIT_PROTOCOL) {
    super(message);
    this.exitCode = exitCode;
  }
}

function isPlainObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

/** Checks one type descriptor from the signature and returns it unchanged. */
function checkType(descriptor) {
  if (!isPlainObject(descriptor)) {
    throw new DriverFailure('a type in the signature is not an object');
  }
  if (!KINDS.has(descriptor.kind)) {
    throw new DriverFailure(`unknown type kind ${JSON.stringify(descriptor.kind)} in the signature`);
  }
  if (descriptor.nullable !== undefined && typeof descriptor.nullable !== 'boolean') {
    throw new DriverFailure('the nullable flag must be true or false');
  }
  if (descriptor.kind === 'list') {
    checkType(descriptor.items);
  }
  if (descriptor.kind === 'tuple') {
    if (!Array.isArray(descriptor.items)) {
      throw new DriverFailure('a tuple type needs a list of item types');
    }
    descriptor.items.forEach(checkType);
  }
  return descriptor;
}

/** Reads and checks the signature file the judge wrote into the sandbox. */
function loadSignature() {
  let data;
  try {
    data = JSON.parse(fs.readFileSync(SIGNATURE_PATH, 'utf8'));
  } catch (error) {
    throw new DriverFailure(`cannot read the signature: ${error.message}`);
  }
  if (!isPlainObject(data) || data.protocol_version !== PROTOCOL_VERSION) {
    throw new DriverFailure('the signature has an unsupported protocol version');
  }
  if (typeof data.function !== 'string' || !IDENTIFIER.test(data.function)) {
    throw new DriverFailure('the signature has no valid function name');
  }
  if (!Array.isArray(data.parameters)) {
    throw new DriverFailure('the signature has no parameter list');
  }
  const parameters = data.parameters.map((parameter) => {
    if (!isPlainObject(parameter)) {
      throw new DriverFailure('a parameter in the signature is not an object');
    }
    return checkType(parameter.type);
  });
  return { functionName: data.function, parameters, returns: checkType(data.returns) };
}

/** True when a JSON-safe value matches a type descriptor. It never converts the value. */
function conforms(value, descriptor) {
  if (value === null) {
    return descriptor.kind === 'null' || descriptor.nullable === true;
  }
  switch (descriptor.kind) {
    case 'int':
      return Number.isSafeInteger(value);
    case 'float':
      return typeof value === 'number' && Number.isFinite(value);
    case 'bool':
      return typeof value === 'boolean';
    case 'string':
      return typeof value === 'string' && value.isWellFormed();
    case 'list':
      return Array.isArray(value) && conformsEach(value, descriptor.items);
    case 'tuple':
      return (
        Array.isArray(value) &&
        value.length === descriptor.items.length &&
        descriptor.items.every((itemType, index) => conforms(value[index], itemType))
      );
    default:
      return false;
  }
}

/** Every item of a list conforms to one item type. Holes and `undefined` never conform. */
function conformsEach(items, itemType) {
  for (let index = 0; index < items.length; index += 1) {
    if (!conforms(items[index], itemType)) {
      return false;
    }
  }
  return true;
}

/** Parses one call line and returns its id and argument list. */
function parseCall(line, signature) {
  if (line.trim() === '') {
    throw new DriverFailure('a blank call line is refused');
  }
  let call;
  try {
    call = JSON.parse(line);
  } catch (error) {
    throw new DriverFailure('the call line is not one JSON value');
  }
  const keys = isPlainObject(call) ? Object.keys(call).sort() : [];
  if (keys.length !== 2 || keys[0] !== 'args' || keys[1] !== 't') {
    throw new DriverFailure('a call must have exactly the keys t and args');
  }
  if (!Number.isSafeInteger(call.t) || call.t < 0) {
    throw new DriverFailure('the call id must be a non-negative integer');
  }
  const { args } = call;
  if (!Array.isArray(args) || args.length !== signature.parameters.length) {
    throw new DriverFailure('the argument count does not match the signature');
  }
  for (let index = 0; index < args.length; index += 1) {
    if (!conforms(args[index], signature.parameters[index])) {
      throw new DriverFailure(`argument ${index} does not match the signature`);
    }
  }
  return { id: call.t, args };
}

/** Finds the function the signature names: the named export first, then a callable module. */
function resolveFunction(moduleExports, name) {
  if (isPlainObject(moduleExports) && typeof moduleExports[name] === 'function') {
    return moduleExports[name];
  }
  if (typeof moduleExports === 'function') {
    return moduleExports;
  }
  throw new DriverFailure(`the solution does not define ${name}`);
}

/** The wrapper functions below live for the process's lifetime. A solution may cache one
 * while it loads, so they consult the call currently owning the capture instead of closing
 * over one call's buffer.
 */
function fsData(data, rest) {
  if (typeof data === 'string') {
    const encoding = rest.findLast((part) => typeof part === 'string');
    return Buffer.from(data, encoding);
  }
  const buffer = Buffer.isBuffer(data)
    ? data
    : Buffer.from(data.buffer || data, data.byteOffset || 0, data.byteLength || data.length);
  const offset = typeof rest[0] === 'number' ? rest[0] : 0;
  const length = typeof rest[1] === 'number' ? rest[1] : buffer.length - offset;
  return buffer.subarray(offset, offset + length);
}

process.stdout.write = (chunk, encoding, callback) => {
  const done = typeof encoding === 'function' ? encoding : callback;
  if (done) done();
  return true;
};
process.stderr.write = (chunk, encoding, callback) => {
  const bytes = Buffer.isBuffer(chunk)
    ? chunk
    : Buffer.from(String(chunk), typeof encoding === 'string' ? encoding : undefined);
  if (activeCapture) activeCapture.push(bytes);
  return writeErr(chunk, encoding, callback);
};
fs.writeSync = (fd, data, ...rest) => {
  if (fd !== 1 && fd !== 2) return writeFsSync(fd, data, ...rest);
  const bytes = fsData(data, rest);
  if (fd === 1) return bytes.length;
  if (activeCapture) activeCapture.push(bytes);
  return writeFsSync(fd, data, ...rest);
};
fs.write = (fd, data, ...rest) => {
  if (fd !== 1 && fd !== 2) return writeFs(fd, data, ...rest);
  const callback = rest[rest.length - 1];
  const bytes = fsData(data, typeof callback === 'function' ? rest.slice(0, -1) : rest);
  if (fd === 1) {
    if (typeof callback === 'function') callback(null, bytes.length, data);
    else writeFs(fd, data, ...rest);
    return;
  }
  if (activeCapture) activeCapture.push(bytes);
  writeFs(fd, data, ...rest);
};
process.exit = () => {
  throw new DriverFailure('the solution called process.exit', EXIT_USER);
};

/** Runs `run` with stdout discarded and bounded stderr captured for the answer. */
function withCapturedOutput(run) {
  const stderr = [];
  let captured = 0;
  let truncated = false;
  activeCapture = {
    push(data) {
      const room = STDERR_CAP - captured;
      if (room > 0) {
        stderr.push(data.subarray(0, Math.min(room, data.length)));
        captured += Math.min(room, data.length);
      }
      truncated ||= data.length > room;
    }
  };
  const rendered = () =>
    Buffer.concat(stderr).toString('utf8') + (truncated ? STDERR_TRUNCATED : '');
  return Promise.resolve()
    .then(run)
    .finally(() => {
      activeCapture = null;
    })
    .then((out) => ({ out, stderr: rendered() }));
}

/** Writes an error's stack to stderr, so the cause is visible without touching stdout. */
function reportError(error) {
  process.stderr.write(`${error && error.stack ? error.stack : String(error)}\n`);
}

/** Loads the solution once and returns the function named in the signature. */
async function loadFunction(signature) {
  let loaded;
  try {
    ({ out: loaded } = await withCapturedOutput(() => require(SOLUTION_PATH)));
  } catch (error) {
    // The driver line goes first: the judge reads the last stderr line as
    // the exception diagnostic, so the stack tail must stay last.
    process.stderr.write('driver: the solution failed while loading\n');
    reportError(error);
    exitProcess(EXIT_PROTOCOL);
  }
  return resolveFunction(loaded, signature.functionName);
}

/** Calls the user's function, awaiting a Promise result and captured stderr. */
async function callSolution(fn, args) {
  try {
    return await withCapturedOutput(() => fn(...args));
  } catch (error) {
    if (error instanceof DriverFailure) {
      throw error;
    }
    // Same ordering: `driver:` diagnostic first, captured output, then stack tail.
    process.stderr.write('driver: the solution raised an exception\n');
    reportError(error);
    exitProcess(EXIT_USER);
  }
}

/** Parses the command line. Only `--mode normal` and `--mode bench` are accepted. */
function parseArguments(argv) {
  if (argv.length === 0) {
    return;
  }
  if (argv.length !== 2 || argv[0] !== '--mode' || !MODES.has(argv[1])) {
    throw new DriverFailure('usage: driver.js [--mode normal|bench]');
  }
}

/** Answers calls until the judge closes stdin. Returns the exit code. */
async function serve(fn, signature, answer) {
  const lines = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
  for await (const line of lines) {
    // Marks this call's process-stderr boundary. Successful user stderr travels in
    // the answer instead, so delayed process output can be discarded safely.
    process.stderr.write(CALL_STDERR_MARKER);
    const { id, args } = parseCall(line, signature);
    const { out, stderr } = await callSolution(fn, args);
    if (!conforms(out, signature.returns)) {
      throw new DriverFailure('the return value is not JSON-safe for the signature');
    }
    const answerLine = stderr ? { t: id, out, stderr } : { t: id, out };
    answer(`${JSON.stringify(answerLine)}\n`);
  }
  return 0;
}

async function main(argv) {
  parseArguments(argv);
  const signature = loadSignature();
  const fn = await loadFunction(signature);
  // Captured before any call discards stdout, so answers always reach the judge.
  const answer = writeOut;
  return serve(fn, signature, answer);
}

main(process.argv.slice(2)).then(
  (code) => {
    process.exitCode = code;
  },
  (error) => {
    const code = error instanceof DriverFailure ? error.exitCode : EXIT_PROTOCOL;
    const message = error instanceof DriverFailure ? error.message : String(error);
    process.stderr.write(`driver: ${message}\n`, () => exitProcess(code));
  }
);
