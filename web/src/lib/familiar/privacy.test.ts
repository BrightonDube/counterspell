// Privacy and safety scan for the Familiar (efficiency-familiar T43, R11.4, R12).
// Speech text, headlines and popover data come from the server and from an AI provider, so
// no Familiar source file may put them into the page as HTML. Each test names the break it catches.
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const HERE = dirname(fileURLToPath(import.meta.url));

/** Every non-test `.svelte` and `.ts` file under `dir`, with its text. */
function sources(dir: string): { path: string; text: string }[] {
  const found: { path: string; text: string }[] = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const { name } = entry;
    const full = join(dir, name);
    if (entry.isDirectory()) {
      found.push(...sources(full));
    } else if (
      entry.isFile() &&
      /\.(svelte|ts)$/.test(name) &&
      !/\.test\.ts$/.test(name)
    ) {
      found.push({
        path: relative(HERE, full),
        text: readFileSync(full, 'utf8')
      });
    }
  }
  return found;
}

/** Ways to turn a string into live markup. */
const HTML_SINKS = [
  /\{@html/,
  /\binnerHTML\b/,
  /\bouterHTML\b/,
  /\binsertAdjacentHTML\b/,
  /\bdocument\.write\b/,
  /\bsrcdoc\b/,
  /\bcreateContextualFragment\b/
];

describe('Familiar sources', () => {
  const files = sources(HERE);

  // Breaks if the scan stops finding files, which would make every check below pass for nothing.
  it('finds the mascot, the bubble and the state files', () => {
    const names = files.map((f) => f.path);
    expect(names).toContain(join('mascot', 'Mascot.svelte'));
    expect(names).toContain(join('bubble', 'SpeechBubble.svelte'));
    expect(names).toContain('Familiar.svelte');
    expect(names).toContain('familiar-state.ts');
  });

  // Breaks if a component renders server or AI text as HTML (script injection through a bubble).
  it('never turns text into HTML', () => {
    const offenders = files
      .filter((file) => HTML_SINKS.some((sink) => sink.test(file.text)))
      .map((file) => file.path);
    expect(offenders).toEqual([]);
  });

  // The mascot parses its own validated SVG art with DOMParser. That is allowed only as SVG:
  // parsing text as HTML would be a sink for server or AI text. Breaks if any other type is used.
  it('uses DOMParser only to parse SVG art', () => {
    const calls = files.flatMap((file) =>
      [...file.text.matchAll(/parseFromString\([^)]*\)/g)].map(
        (m) => `${file.path}: ${m[0]}`
      )
    );
    expect(calls.length).toBeGreaterThan(0);
    for (const call of calls) expect(call, call).toContain("'image/svg+xml'");
  });

  // Breaks if the scan's patterns stop matching real sinks.
  it('the scan itself recognises each sink', () => {
    for (const sample of [
      '{@html body}',
      'el.innerHTML = body',
      'el.outerHTML = body',
      'el.insertAdjacentHTML("beforeend", body)',
      'document.write(body)',
      'frame.srcdoc = body'
    ]) {
      expect(
        HTML_SINKS.some((sink) => sink.test(sample)),
        sample
      ).toBe(true);
    }
  });
});
