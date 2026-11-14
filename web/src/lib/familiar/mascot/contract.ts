/**
 * The SVG ids the mascot binds to. Any artwork that carries these ids works, so the drawing can
 * change without changing the code. See the design's "Asset contract" section.
 */
export const REQUIRED_IDS = [
	'fam-body',
	'fam-glow-stop-inner',
	'fam-glow-stop-outer',
	'fam-eyes-1',
	'fam-eyes-2',
	'fam-eyes-3',
	'fam-eyes-4',
	'fam-eyes-5',
	'fam-mouth-1',
	'fam-mouth-2',
	'fam-mouth-3',
	'fam-mouth-4',
	'fam-mouth-5',
	'fam-cracks',
	'fam-fx'
] as const;

/** The result of checking an SVG against the contract. */
export type ContractResult = { ok: boolean; missing: string[] };

/**
 * Checks that an SVG text declares every required id. It matches whole attribute values, so
 * `fam-fx-extra` does not satisfy `fam-fx`.
 */
export function validate(svgText: string): ContractResult {
	const declared = new Set<string>();
	for (const match of svgText.matchAll(/\sid="([^"]+)"/g)) {
		declared.add(match[1]);
	}
	const missing = REQUIRED_IDS.filter((id) => !declared.has(id));
	return { ok: missing.length === 0, missing: [...missing] };
}
