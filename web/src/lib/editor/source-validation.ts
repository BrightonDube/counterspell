// Advisory size checks for the editor. The server repeats every check, so these only give early feedback.
import { byteLength } from '../contract';

export type SizeCheck =
  | { ok: true; bytes: number }
  | { ok: false; bytes: number; limit: number; message: string };

/** Compares the UTF-8 size of a source with the language's byte limit. Lines are never counted. */
export function checkSourceSize(source: string, limit: number): SizeCheck {
  const bytes = byteLength(source);
  if (bytes <= limit) {
    return { ok: true, bytes };
  }
  return {
    ok: false,
    bytes,
    limit,
    message: `This solution is ${bytes} bytes. The limit for this language is ${limit} bytes.`
  };
}

/** The part of the Clipboard API the copy fallback needs. */
export interface ClipboardLike {
  writeText(text: string): Promise<void>;
}

/** Copies the source as plain text. Returns false when there is no clipboard or it refuses the write. */
export async function copySource(
  source: string,
  clipboard: ClipboardLike | undefined
): Promise<boolean> {
  if (!clipboard) return false;
  try {
    await clipboard.writeText(source);
    return true;
  } catch {
    return false;
  }
}
