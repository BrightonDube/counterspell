// View model for the workspace console. Only public sample detail reaches the page; hidden
// tests have already been reduced to ordinal, verdict and time by the parser.
import type { TerminalPayload } from '../runs/reducer';
import type { PublicTest } from '../runs/types';
import { OUTPUT_DISPLAY_CHARS } from './test-rows';

export { OUTPUT_DISPLAY_CHARS };

export interface ConsoleEntry {
  id: string;
  heading: string;
  text: string;
}

export interface ConsoleInput {
  tests: readonly PublicTest[];
  terminal?: TerminalPayload;
}

const TRUNCATED_MARK = '… (truncated)';

function truncate(text: string): string {
  if (text.length <= OUTPUT_DISPLAY_CHARS) return text;
  return text.slice(0, OUTPUT_DISPLAY_CHARS) + TRUNCATED_MARK;
}

/** Builds compile and sample output/stderr blocks in test order. */
export function buildConsoleEntries(input: ConsoleInput): ConsoleEntry[] {
  const entries: ConsoleEntry[] = [];
  if (input.terminal?.kind === 'done' && input.terminal.compileOutput) {
    entries.push({
      id: 'compile',
      heading: 'Compile output',
      text: truncate(input.terminal.compileOutput)
    });
  }
  const samples = input.tests
    .filter((test) => test.sample)
    .sort((a, b) => a.ordinal - b.ordinal);
  for (const test of samples) {
    if (test.got) {
      entries.push({
        id: `test-${test.ordinal}-output`,
        heading: `Sample ${test.ordinal} output`,
        text: truncate(test.got)
      });
    }
    if (test.stderr) {
      entries.push({
        id: `test-${test.ordinal}-stderr`,
        heading: `Sample ${test.ordinal} stderr`,
        text: truncate(test.stderr)
      });
    }
  }
  return entries;
}
