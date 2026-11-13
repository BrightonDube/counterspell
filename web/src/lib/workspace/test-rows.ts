// View model for the test panel. A hidden row is built from ordinal, verdict and time only, so hidden
// input, expected output and program output cannot reach the view. Labels are plain text with an icon,
// so status is never shown by colour alone.
import type { TerminalPayload } from '../runs/reducer';
import type { PublicTest, RunStatus, TestVerdict } from '../runs/types';

/** Longest sample value shown in full. Longer values are cut and marked. */
export const OUTPUT_DISPLAY_CHARS = 2000;
const TRUNCATED_MARK = '… (truncated)';

const VERDICT_LABELS: Record<TestVerdict, string> = {
  pass: 'Passed',
  wrong_answer: 'Wrong answer',
  runtime_error: 'Runtime error',
  time_limit: 'Time limit exceeded',
  memory_limit: 'Memory limit exceeded',
  output_limit: 'Output limit exceeded'
};

const STATUS_LABELS: Record<RunStatus, string> = {
  queued: 'Queued',
  claimed: 'Starting',
  compiling: 'Compiling',
  running: 'Running',
  ghost_hunting: 'Hunting a ghost',
  completed: 'Finished',
  failed: 'Failed',
  expired: 'Expired',
  internal_error: 'Internal error (not a wrong answer)'
};

const DONE_LABELS: Record<string, string> = {
  accepted: 'Accepted',
  wrong_answer: 'Wrong answer',
  runtime_error: 'Runtime error',
  time_limit: 'Time limit exceeded',
  memory_limit: 'Memory limit exceeded',
  output_limit: 'Output limit exceeded',
  compile_error: 'Compile error'
};

export interface HiddenRowView {
  kind: 'hidden';
  ordinal: number;
  label: string;
  icon: string;
  timeLabel: string;
}

export interface SampleRowView {
  kind: 'sample';
  ordinal: number;
  label: string;
  icon: string;
  timeLabel: string;
  input: string;
  expected: string;
  got: string;
}

export type TestRowView = HiddenRowView | SampleRowView;

function truncate(text: string): string {
  if (text.length <= OUTPUT_DISPLAY_CHARS) return text;
  return text.slice(0, OUTPUT_DISPLAY_CHARS) + TRUNCATED_MARK;
}

/** Turns one public test row into the text the panel shows. */
export function rowView(test: PublicTest): TestRowView {
  const shared = {
    ordinal: test.ordinal,
    label: VERDICT_LABELS[test.verdict],
    icon: test.verdict === 'pass' ? '✓' : '✗',
    timeLabel: `${test.timeMs} ms`
  };
  if (!test.sample) return { kind: 'hidden', ...shared };
  return {
    kind: 'sample',
    ...shared,
    input: truncate(test.input),
    expected: truncate(test.expected),
    got: truncate(test.got)
  };
}

/** The plain-English label for a run status. */
export function statusLabel(status: RunStatus): string {
  return STATUS_LABELS[status];
}

/** The plain-English result of a finished run. An internal error is never shown as a wrong answer. */
export function runOutcomeLabel(terminal: TerminalPayload): string {
  if (terminal.kind === 'done')
    return DONE_LABELS[terminal.verdict] ?? 'Finished';
  return terminal.code === 'internal_error'
    ? 'Internal error (not a wrong answer)'
    : 'Run failed';
}
