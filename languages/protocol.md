# Driver protocol, version 1

This document defines the line protocol between the trusted judge and a language driver
inside a sandbox. Each driver template under `languages/drivers/<language>/` implements it.
The judge's parser is `crates/judge/src/sandbox/driver_protocol.rs`. A test checks that the
examples below parse, so this document and the code cannot drift apart.

## Roles

- **The judge (trusted)** owns the test inputs, the expected outputs, the comparator, the
  per-call deadlines, the timing and the final verdict.
- **The driver** loads the user's solution once, calls the user's function for each request
  and writes back the raw return value. Nothing else.
- **The user's solution** only receives arguments and returns a value. Anything it prints is
  untrusted data.

**Drivers never emit verdicts, expected values, timings, complexity classes or Efficiency
Familiar moods.** Output that looks like a verdict, for example `{"verdict": "accepted"}`,
is a wrong answer as far as the judge is concerned. The judge only compares it.

## Files in the sandbox

| Path | Written by | Contents |
|---|---|---|
| `/box/solution.<ext>` | judge | the user's source, the only user-supplied file |
| `/box/driver.<ext>` | judge | the read-only driver template for the language |
| `/box/signature.json` | judge | the public signature descriptor (below) |

The signature descriptor holds only public signature metadata. It never holds test cases,
expected outputs, hidden data, the comparator, reference code or time limits.

## Signature descriptor

```json
{
  "protocol_version": 1,
  "function": "two_sum",
  "parameters": [
    {"name": "nums", "type": {"kind": "list", "items": {"kind": "int"}}},
    {"name": "target", "type": {"kind": "int"}}
  ],
  "returns": {"kind": "list", "items": {"kind": "int"}}
}
```

The type kinds are `int`, `float`, `bool`, `string`, `list` (with `items`), `tuple` (with
`items`, a JSON array of fixed length), and `null`. A `nullable` flag may wrap any kind
where the problem allows it. Later kinds such as `TreeNode` and `ListNode` are reserved
and not part of version 1.

## Framing

- **Call (judge to driver, stdin):** one UTF-8 JSON object per line, ending in `\n`.
  `t` is a non-negative integer call ID. `args` is an array with one JSON value per parameter,
  in signature order.
- **Answer (driver to judge, stdout):** one UTF-8 JSON object per line, ending in `\n`,
  with exactly two keys, `t` and `out`. `t` must echo the call ID. `out` is the JSON-safe
  return value.
- The judge sends a call only after the previous answer has arrived. It starts the deadline
  before writing the call and stops it when the full answer line has been read.
- The driver flushes stdout after every answer.

Call example:

```json
{"t": 0, "args": [[2, 7, 11, 15], 9]}
```

Answer example:

```json
{"t": 0, "out": [0, 1]}
```

Answer with a null return, when the signature allows null:

```json
{"t": 1, "out": null}
```

Each of these lines is refused:

```text
{"t": 0, "out": 1, "verdict": "accepted"}      extra key: refused
{"t": 0}                                       no out: refused
{"t": 9, "out": 1}                             call 9 answered to call 0: refused
(a blank line)                                 refused, so call order stays unambiguous
```

## Supported JSON-safe values

The driver accepts and returns only these values:

- integers that fit in the exact range of the language's JSON number handling (JavaScript:
  within ±2^53 - 1). Launch packs declare their ranges so that no value falls outside it;
- finite floats. `NaN`, `Infinity` and `-Infinity` are refused;
- booleans, strings (valid Unicode), arrays, nested arrays, and JSON objects where a pack
  declares them;
- `null`, only where the signature allows it.

The driver does not coerce values. It never turns a boolean into a number, `null` into an
empty string, or a float into an integer. Object key order is not a semantic value; the
judge's comparator compares parsed JSON values.

## Output and stderr rules

- **stdout** carries protocol answer lines only. Output the user's function prints while it
  runs is captured by the driver, so it cannot corrupt the answer stream.
- **stderr** carries diagnostics, such as a setup failure or a traceback. It is capped by the
  sandbox output limit. It is never parsed as an answer and never becomes a verdict.
- Output that bypasses that capture and breaks the framing is a protocol failure. The judge
  classifies it from the process outcome. It is never read as a wrong answer from the user.

## Errors

The driver does not manufacture an answer when something fails. It exits non-zero, or leaves
its answer line unwritten, in these cases:

| Condition | Driver behaviour | Judge classification |
|---|---|---|
| user function raises | diagnostic on stderr, no answer | runtime error (from the process outcome) |
| return value is not JSON-safe | diagnostic on stderr, no answer | invalid answer |
| argument does not match the signature | diagnostic on stderr, no answer | protocol failure |
| malformed input line, or a blank line | diagnostic on stderr, exit | protocol failure |
| process exits or stops responding | none | process failure or time limit |

None of these becomes Accepted. Infrastructure failures are reported as internal errors, not
as the user's wrong answer.

## Source export conventions

- **Python:** the solution defines a top-level function with the name in
  `signature.json`. The driver loads `/box/solution.py` once, without importing the reference.
- **JavaScript (CommonJS):** the solution assigns the function to `module.exports`, either
  directly (`module.exports = function two_sum(nums, target) {...}`) or as a named property
  (`module.exports = { two_sum }`). The driver checks the named export first, then accepts a
  callable default module export when the declared function name is not an export.

## Bench mode

`--mode bench` is passed through the language's `bench_args` in `languages.toml`. It
selects a driver that loads the solution once and accepts repeated calls. The answer
framing is identical. Bench mode never measures time, chooses sizes or classifies
complexity. The judge owns all timing.

## Versioning

`protocol_version` is `1` in `signature.json` and in every `languages.toml` entry. The judge
refuses any other version. A change to the framing or the value rules needs a new version
and a matching driver for each enabled language.
