/**
 * The SVG ids the mascot binds to. The realistic art (approved 15 Nov 2026) shows one of five
 * painted mood images at a time, so the contract asks for five image layers and no carved eye or
 * mouth paths. The drawing can change without changing the code. See the design's "Asset contract"
 * and its 15 Nov 2026 amendment.
 */
export const REQUIRED_IDS = [
  'fam-body',
  'fam-glow-stop-inner',
  'fam-glow-stop-outer',
  'fam-mood-1',
  'fam-mood-2',
  'fam-mood-3',
  'fam-mood-4',
  'fam-mood-5',
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
