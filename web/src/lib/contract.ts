// Shared guards for API and SSE payloads. Every value from the server passes through these checks
// before a component can see it. A malformed or unexpected field becomes a ContractError, never a
// guessed value.

/** The largest text the browser displays or stores from one field, in UTF-8 bytes. */
export const DISPLAY_CAP_BYTES = 64 * 1024;

/** The largest JSON value the browser accepts for an input, expected or measured value, in bytes. */
export const MAX_JSON_BYTES = 16 * 1024;

/** A value that can be written as JSON. Nothing else is accepted from the server. */
export type Json =
  null | boolean | number | string | Json[] | { [key: string]: Json };

/** A parsed JSON object with unknown fields, read one field at a time by the guards below. */
export type RawRecord = Record<string, unknown>;

/** Raised when a server value does not match its documented contract. */
export class ContractError extends Error {
  readonly path: string;

  constructor(path: string, reason: string) {
    super(`${path}: ${reason}`);
    this.name = 'ContractError';
    this.path = path;
  }
}

/** Returns the UTF-8 length of a string, which is what the server and the byte limits count. */
export function byteLength(text: string): number {
  return new TextEncoder().encode(text).length;
}

/** Throws when a string is longer than the given UTF-8 byte cap. */
export function assertBytes(
  text: string,
  maxBytes: number,
  path: string
): void {
  if (byteLength(text) > maxBytes) {
    throw new ContractError(path, `text is larger than ${maxBytes} bytes`);
  }
}

/** Returns the value as an object, or throws if it is an array, null or a scalar. */
export function asRecord(value: unknown, path: string): RawRecord {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    throw new ContractError(path, 'expected an object');
  }
  return value as RawRecord;
}

/** Checks that an array entry is a string within the cap, and returns it. */
export function asString(
  value: unknown,
  path: string,
  maxBytes: number = DISPLAY_CAP_BYTES
): string {
  if (typeof value !== 'string') {
    throw new ContractError(path, 'expected a string');
  }
  assertBytes(value, maxBytes, path);
  return value;
}

/** Reads a required string, bounded by the display cap unless a smaller cap is given. */
export function readString(
  source: RawRecord,
  key: string,
  path: string,
  maxBytes: number = DISPLAY_CAP_BYTES
): string {
  const value = source[key];
  if (typeof value !== 'string') {
    throw new ContractError(`${path}.${key}`, 'expected a string');
  }
  assertBytes(value, maxBytes, `${path}.${key}`);
  return value;
}

/** Reads an optional string. Missing and null both give undefined. */
export function readOptionalString(
  source: RawRecord,
  key: string,
  path: string,
  maxBytes: number = DISPLAY_CAP_BYTES
): string | undefined {
  if (source[key] === undefined || source[key] === null) return undefined;
  return readString(source, key, path, maxBytes);
}

/** Reads a required integer within an inclusive range. */
export function readInt(
  source: RawRecord,
  key: string,
  path: string,
  min: number,
  max: number
): number {
  const value = source[key];
  if (
    typeof value !== 'number' ||
    !Number.isInteger(value) ||
    value < min ||
    value > max
  ) {
    throw new ContractError(
      `${path}.${key}`,
      `expected an integer from ${min} to ${max}`
    );
  }
  return value;
}

/** Reads an optional integer within an inclusive range. Missing and null both give undefined. */
export function readOptionalInt(
  source: RawRecord,
  key: string,
  path: string,
  min: number,
  max: number
): number | undefined {
  if (source[key] === undefined || source[key] === null) return undefined;
  return readInt(source, key, path, min, max);
}

/** Reads a required boolean. */
export function readBool(
  source: RawRecord,
  key: string,
  path: string
): boolean {
  const value = source[key];
  if (typeof value !== 'boolean') {
    throw new ContractError(`${path}.${key}`, 'expected a boolean');
  }
  return value;
}

/** Reads a required value that must be one of the allowed strings. */
export function readEnum<T extends string>(
  source: RawRecord,
  key: string,
  path: string,
  allowed: readonly T[]
): T {
  const value = source[key];
  if (
    typeof value !== 'string' ||
    !(allowed as readonly string[]).includes(value)
  ) {
    throw new ContractError(
      `${path}.${key}`,
      `expected one of ${allowed.join(', ')}`
    );
  }
  return value as T;
}

/** Reads an array of at most `maxItems` entries, parsing each entry with `item`. */
export function readArray<T>(
  source: RawRecord,
  key: string,
  path: string,
  maxItems: number,
  item: (value: unknown, path: string) => T
): T[] {
  const value = source[key];
  if (!Array.isArray(value)) {
    throw new ContractError(`${path}.${key}`, 'expected an array');
  }
  if (value.length > maxItems) {
    throw new ContractError(
      `${path}.${key}`,
      `expected at most ${maxItems} items`
    );
  }
  return value.map((entry, index) => item(entry, `${path}.${key}[${index}]`));
}

/** Checks that a value is plain JSON and that its serialised form fits the JSON cap. */
export function readJson(source: RawRecord, key: string, path: string): Json {
  const value = source[key];
  if (value === undefined) {
    throw new ContractError(`${path}.${key}`, 'expected a JSON value');
  }
  if (!isJson(value)) {
    throw new ContractError(`${path}.${key}`, 'expected plain JSON');
  }
  const serialised = JSON.stringify(value);
  if (byteLength(serialised) > MAX_JSON_BYTES) {
    throw new ContractError(
      `${path}.${key}`,
      `JSON is larger than ${MAX_JSON_BYTES} bytes`
    );
  }
  return value;
}

function isJson(value: unknown): value is Json {
  if (value === null) return true;
  switch (typeof value) {
    case 'string':
    case 'boolean':
      return true;
    case 'number':
      return Number.isFinite(value);
    case 'object':
      if (Array.isArray(value)) return value.every(isJson);
      return Object.values(value as RawRecord).every(isJson);
    default:
      return false;
  }
}
