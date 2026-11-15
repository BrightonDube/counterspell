// Checks the approved font wiring in app.css (ide-workspace T05).
// Geist and JetBrains Mono are OFL-1.1 and must load from the pinned packages,
// not from a remote CDN.
import { readFileSync, existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const here = dirname(fileURLToPath(import.meta.url));
const css = readFileSync(resolve(here, 'app.css'), 'utf8');

describe('app.css font wiring (T05)', () => {
	it('declares a Geist @font-face whose woff2 file exists in the pinned package', () => {
		const block = css.match(/@font-face\s*\{[^}]*font-family:\s*'?Geist'?[^}]*\}/);
		expect(block, 'Geist @font-face block').not.toBeNull();

		const url = block?.[0].match(/url\('([^']+\.woff2)'\)/);
		expect(url, 'Geist woff2 url').not.toBeNull();
		const path = url?.[1] ?? '';
		expect(path).not.toMatch(/^https?:/);
		expect(existsSync(resolve(here, path))).toBe(true);
	});

	it('imports JetBrains Mono weights 400 to 700 from the pinned fontsource package', () => {
		for (const weight of [400, 500, 600, 700]) {
			expect(css).toContain(`@import '@fontsource/jetbrains-mono/${weight}.css';`);
		}
	});

	it('keeps the sans and mono tokens pointing at the approved families', () => {
		expect(css).toMatch(/--font-sans:\s*'Geist'/);
		expect(css).toMatch(/--font-mono:\s*'JetBrains Mono'/);
	});
});
