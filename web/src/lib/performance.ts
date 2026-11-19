// Browser timing marks for the workspace (ide-workspace R15.5). A mark is only a fixed name: no
// detail object is ever attached, so no source text, user id or run id can reach the
// Performance timeline.

/** Every mark the workspace records, in the order a run normally produces them. */
export const WORKSPACE_MARKS = [
  'workspace-shell-interactive',
  'problem-detail-loaded',
  'monaco-ready',
  'run-first-event',
  'run-terminal',
  'ghost-playback-start',
  'ghost-playback-end'
] as const;

export type WorkspaceMark = (typeof WORKSPACE_MARKS)[number];

/** The one Performance method the marks need. Tests pass a recorder. */
export type MarkSink = { mark(name: string): unknown } | undefined;

function defaultSink(): MarkSink {
  return typeof performance === 'undefined' ? undefined : performance;
}

/** Records one named mark. Does nothing where the Performance API is missing. */
export function markWorkspace(
  name: WorkspaceMark,
  sink: MarkSink = defaultSink()
): void {
  sink?.mark(name);
}

/** Marks the first streamed event and the terminal result once per run. */
export function createRunMarks(sink: MarkSink = defaultSink()): {
  event(runId: string): void;
  terminal(runId: string): void;
} {
  let eventRun = '';
  let terminalRun = '';
  return {
    event(runId) {
      if (runId === eventRun) return;
      eventRun = runId;
      markWorkspace('run-first-event', sink);
    },
    terminal(runId) {
      if (runId === terminalRun) return;
      terminalRun = runId;
      markWorkspace('run-terminal', sink);
    }
  };
}

/** How long the shrink replay plays. A static plan plays nothing, so it takes no time. */
export function playbackDurationMs(plan: {
  mode: 'animated' | 'static';
  stepCount: number;
  stepDelayMs: number;
}): number {
  return plan.mode === 'static' ? 0 : plan.stepCount * plan.stepDelayMs;
}

/**
 * Marks the start of a ghost replay now and its end once `durationMs` has passed. Returns a
 * cancel function for when the panel goes away first.
 */
export function scheduleGhostMarks(
  durationMs: number,
  sink: MarkSink = defaultSink()
): () => void {
  markWorkspace('ghost-playback-start', sink);
  if (durationMs <= 0) {
    markWorkspace('ghost-playback-end', sink);
    return () => {};
  }
  const timer = setTimeout(
    () => markWorkspace('ghost-playback-end', sink),
    durationMs
  );
  return () => clearTimeout(timer);
}
