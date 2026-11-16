// The "How does it judge me?" view-model. Pure: it turns the stored `complexity` section of the run
// read model into the text and numbers the popover shows. It invents nothing. A missing section is
// reported as not measured.

/** One stored timing point: the input size and the median time for it. */
export interface TimingPoint {
	n: number;
	median_ns: number;
}

/** The `complexity` section of `GET /api/runs/{id}`, as the server sends it. */
export interface ComplexitySection {
	class: string | null;
	confidence: 'low' | 'medium' | 'high';
	reference: string;
	plausible: string[];
	points: TimingPoint[];
	slope: number | null;
	slope_se: number | null;
	residuals: Record<string, number>;
	ratio_vs_baseline: number | null;
	fit_version: string;
}

/** A point in the view, with the names the popover uses. */
export interface ExplainPoint {
	n: number;
	medianNs: number;
}

/** One model's residual, sorted so the best fit comes first. */
export interface ResidualRow {
	model: string;
	label: string;
}

/** Everything the popover shows. Strings are plain text; the component renders them as text. */
export interface Explanation {
	measured: boolean;
	summary: string;
	points: ExplainPoint[];
	slopeText: string;
	confidenceText: string;
	referenceText: string;
	plausibleText: string[];
	residualRows: ResidualRow[];
	ratioText: string | null;
	limits: string[];
}

/** The display name of each class id. These match the Rust `display_name` text. */
const CLASS_NAME: Record<string, string> = {
	'1': 'constant',
	'log n': 'logarithmic',
	n: 'linear',
	'n log n': 'n log n',
	'n^2': 'quadratic',
	'n^3': 'cubic',
	'2^n': 'exponential'
};

/** The limits the design requires the popover to state. They are fixed text, not measured data. */
const LIMITS: string[] = [
	'Times are medians of repeated runs, not single measurements.',
	'Growth is read from size ratios, and the fit must agree before a class is reported.',
	'Constant factors are compared only with a same-language baseline.',
	'The server is shared, so noise can change a timing. Small inputs can hide the true growth.',
	'Some problems have no meaningful growth curve. Those show only a pass or a fail.'
];

function nameOf(classId: string): string {
	return CLASS_NAME[classId] ?? classId;
}

function confidenceTextFor(section: ComplexitySection): string {
	if (section.confidence === 'low') {
		return 'Unsure: the data does not single out one class.';
	}
	return `${section.confidence === 'high' ? 'High' : 'Medium'} confidence.`;
}

/** Builds the popover text from the stored section. `null` means the run was not measured. */
export function explain(section: ComplexitySection | null): Explanation {
	if (section === null) {
		return {
			measured: false,
			summary: 'This run was not timed, so there is no growth reading.',
			points: [],
			slopeText: '',
			confidenceText: '',
			referenceText: '',
			plausibleText: [],
			residualRows: [],
			ratioText: null,
			limits: LIMITS
		};
	}
	const slopeText =
		section.slope === null
			? 'No slope was measured.'
			: `Slope ${section.slope.toFixed(2)} ± ${(section.slope_se ?? 0).toFixed(2)} on a log-log scale.`;
	const residualRows = Object.entries(section.residuals)
		.sort(([, a], [, b]) => a - b)
		.map(([model, value]) => ({ model, label: `${nameOf(model)}: residual ${value.toFixed(2)}` }));
	const plausibleText =
		section.confidence === 'low'
			? section.plausible.map((classId) => `Could be ${nameOf(classId)}`)
			: section.plausible.map((classId) => `Fits ${nameOf(classId)}`);
	return {
		measured: true,
		summary:
			section.class === null
				? 'The fit could not name one class.'
				: `Looks ${nameOf(section.class)}, judged from the stored timings.`,
		points: section.points.map((point) => ({ n: point.n, medianNs: point.median_ns })),
		slopeText,
		confidenceText: confidenceTextFor(section),
		referenceText: `The reference is ${nameOf(section.reference)}.`,
		plausibleText,
		residualRows,
		ratioText:
			section.ratio_vs_baseline === null
				? null
				: `${section.ratio_vs_baseline.toFixed(2)} times the same-language baseline.`,
		limits: LIMITS
	};
}
