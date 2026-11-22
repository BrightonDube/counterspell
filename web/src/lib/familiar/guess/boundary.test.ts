// test-change: R14.1/R14.6 (T48 import boundary for the live typing guess)
//
// The guess reads the editor text locally and can never reach the mood path
// or the network. This test scans the sources: `guess/**` must import nothing
// from `familiar-state`, `mascot`, `$lib/runs` or any network API, and nothing
// in the mood path may import `guess/`.

import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';
import { describe, expect, it } from 'vitest';

const GUESS_DIR = import.meta.dirname;
const FAMILIAR_DIR = join(GUESS_DIR, '..');

function sourcesUnder(dir: string): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) return sourcesUnder(path);
    return /\.(ts|svelte)$/.test(entry) ? [path] : [];
  });
}

function importSpecifiers(source: string): string[] {
  const out: string[] = [];
  for (const match of source.matchAll(
    /(?:import|export)[^'"]*?from\s*['"]([^'"]+)['"]|import\s*\(\s*['"]([^'"]+)['"]/g
  )) {
    out.push(match[1] ?? match[2]);
  }
  return out;
}

const FORBIDDEN_IMPORTS = [
  'familiar-state',
  'mascot',
  '$lib/runs',
  '$lib/api',
  '../api',
  'wails'
];
const NETWORK_APIS = [
  'fetch(',
  'XMLHttpRequest',
  'WebSocket',
  'EventSource',
  'sendBeacon',
  'wails'
];

describe('guess boundary', () => {
  it('guess sources import nothing from the mood path or the network', () => {
    const files = sourcesUnder(GUESS_DIR).filter(
      (path) => !path.endsWith('boundary.test.ts')
    );
    expect(files.length).toBeGreaterThan(0);
    for (const file of files) {
      const rel = relative(FAMILIAR_DIR, file);
      const source = readFileSync(file, 'utf8');
      for (const specifier of importSpecifiers(source)) {
        for (const forbidden of FORBIDDEN_IMPORTS) {
          expect(specifier, `${rel} imports ${specifier}`).not.toContain(
            forbidden
          );
        }
      }
      for (const api of NETWORK_APIS) {
        expect(source, `${rel} uses ${api}`).not.toContain(api);
      }
    }
  });

  it('nothing in the mood path imports the guess', () => {
    const moodPath = sourcesUnder(FAMILIAR_DIR).filter(
      (path) => !path.startsWith(GUESS_DIR)
    );
    for (const file of moodPath) {
      const rel = relative(FAMILIAR_DIR, file);
      for (const specifier of importSpecifiers(readFileSync(file, 'utf8'))) {
        expect(specifier, `${rel} imports ${specifier}`).not.toContain('guess');
      }
    }
  });
});
