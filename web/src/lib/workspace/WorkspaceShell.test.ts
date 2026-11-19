// Component tests for the responsive workspace shell (ide-workspace T09). Each test names the layout break it catches.
import {
  cleanup,
  fireEvent,
  render,
  screen,
  within
} from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { LanguageSummary, ProblemDetail } from '../api/types';

vi.mock('../editor/monaco-loader', () => ({
  CRYPT_THEME_NAME: 'crypt',
  loadMonaco: () => new Promise(() => {})
}));

// The run API is replaced so no test calls the network. The real module is kept for its other exports.
vi.mock('../runs/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../runs/api')>();
  return {
    ...actual,
    startRunAction: vi.fn(() => ({
      key: 'key-1',
      send: () => new Promise<string>(() => {})
    }))
  };
});

import { startRunAction } from '../runs/api';
import { initialRunState, type RunStreamState } from '../runs/reducer';
import { parseRunView } from '../runs/types';
import type { GhostView } from '../ghost/types';
import type { RunSessionDeps } from './run-session';
import WorkspaceShell from './WorkspaceShell.svelte';

afterEach(() => {
  cleanup();
});

/** A minimal wrong-answer ghost, as the trusted judge's stream payload parses it. */
function ghostFixture(): GhostView {
  return {
    kind: 'wrong',
    input: [0, 0],
    expected: 1,
    got: 0,
    explanation: 'You returned 0; expected 1 for [0, 0].',
    budgetExhausted: false,
    shrink: {
      steps: [
        { index: 0, input: [3, 5, 0], evaluations: 1 },
        { index: 1, input: [0, 0], evaluations: 4 }
      ],
      totalSteps: 2,
      omittedSteps: 0,
      stop: 'no_smaller_input'
    },
    provenance: {
      seed: '7',
      generatorVersion: 'gen-1',
      engineVersion: 'eng-1',
      problemVersion: 2,
      imageDigest: 'sha256:abc'
    },
    autopsy: {
      explanation: 'You returned 0; expected 1 for [0, 0].',
      differences: [],
      differencesTruncated: false,
      conceptTags: [],
      authoredMistake: null
    }
  };
}

/** Session deps whose watch replays a finished stream that carries a ghost. */
function ghostSessionDeps(
  stream: RunStreamState,
  runId = 'run-ghost'
): RunSessionDeps {
  return {
    startAction: () => ({
      key: 'ghost-key',
      send: () => Promise.resolve(runId)
    }),
    fetchRun: () =>
      Promise.resolve(
        parseRunView({
          run_id: runId,
          problem: 'two-sum',
          lang: 'python',
          mode: 'run',
          status: 'completed',
          terminal: true,
          verdict: 'wrong_answer',
          attempt: 1,
          created_at: '2026-11-15T10:00:00Z',
          last_seq: 3,
          tests: []
        })
      ),
    watch: (options) => {
      options.onDone(stream);
      return { close() {} };
    }
  };
}

/** Familiar deps that never touch the network and ask for no speech. */
function familiarOff() {
  return {
    loadPreferences: () =>
      Promise.resolve({ muted: false, hidden: false, speech: 'off' as const }),
    savePreferences: () => Promise.resolve({ synced: true }),
    requestSpeech: () =>
      Promise.resolve({ kind: 'unavailable' as const, text: '' })
  };
}

const problem: ProblemDetail = {
  slug: 'two-sum',
  title: 'Two Sum',
  track: { slug: 'arrays', name: 'Crypt of Arrays' },
  concepts: [],
  difficulty: 'easy',
  statementMarkdown: 'Return indices.',
  constraints: [],
  signature: {
    functionName: 'two_sum',
    parameters: [],
    returnType: 'list<int>'
  },
  samples: [],
  starters: {},
  enabledLanguages: ['python']
};

const languages: LanguageSummary[] = [
  {
    id: 'python',
    displayName: 'Python',
    maxSourceBytes: 65536
  }
];

describe('WorkspaceShell', () => {
  // Breaks if a region is unnamed, so a screen-reader user cannot jump between problem, editor and results.
  it('exposes the problem, editor and results as named landmarks', () => {
    render(WorkspaceShell, {
      props: { problem, languages, layout: 'desktop' }
    });
    expect(screen.getByRole('complementary', { name: 'Problem' })).toBeTruthy();
    expect(screen.getByRole('region', { name: 'Editor' })).toBeTruthy();
    expect(screen.getByRole('region', { name: 'Results' })).toBeTruthy();
  });

  // Breaks if the problem column cannot be collapsed, which leaves no room for the editor on a tablet.
  it('collapses and restores the problem column from its toggle', async () => {
    render(WorkspaceShell, {
      props: { problem, languages, layout: 'desktop' }
    });
    const toggle = screen.getByRole('button', { name: 'Hide problem' });
    const aside = screen.getByRole('complementary', { name: 'Problem' });
    expect(toggle.getAttribute('aria-expanded')).toBe('true');

    await fireEvent.click(toggle);
    expect(toggle.getAttribute('aria-expanded')).toBe('false');
    expect(aside.hidden).toBe(true);
    expect(screen.getByRole('button', { name: 'Show problem' })).toBeTruthy();
  });

  // Breaks if a double click on Run queues two runs for one learner action.
  it('sends one run when Run is clicked twice', async () => {
    render(WorkspaceShell, {
      props: { problem, languages, layout: 'desktop' }
    });
    const run = screen.getByRole('button', { name: 'Run samples' });
    await fireEvent.click(run);
    await fireEvent.click(run);
    expect(startRunAction).toHaveBeenCalledTimes(1);
    expect(startRunAction).toHaveBeenCalledWith({
      problem: 'two-sum',
      lang: 'python',
      code: '',
      mode: 'run'
    });
  });

  // Breaks if the tablet layout leaves the problem or results inline, so the editor and
  // controls no longer remain the only persistent regions.
  it('moves the problem and results behind labelled triggers on tablet', () => {
    render(WorkspaceShell, {
      props: { problem, languages, layout: 'tablet' }
    });
    expect(screen.getByRole('region', { name: 'Editor' })).toBeTruthy();
    for (const name of ['Problem', 'Results']) {
      const trigger = screen.getByRole('button', { name });
      expect(trigger.getAttribute('aria-expanded')).toBe('false');
    }
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(screen.queryByRole('complementary')).toBeNull();
  });

  // Breaks if opening a drawer does not move focus inside it, or Escape does not close
  // the drawer and return focus to its trigger.
  it('tablet drawer moves focus inside and Escape returns focus to the trigger', async () => {
    render(WorkspaceShell, {
      props: { problem, languages, layout: 'tablet' }
    });
    const trigger = screen.getByRole('button', { name: 'Problem' });
    trigger.focus();
    await fireEvent.click(trigger);
    expect(trigger.getAttribute('aria-expanded')).toBe('true');

    const drawer = screen.getByRole('dialog', { name: 'Problem' });
    expect(drawer.contains(document.activeElement)).toBe(true);

    await fireEvent.keyDown(drawer, { key: 'Escape' });
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(document.activeElement).toBe(trigger);
  });

  // Breaks if the tablet layout loses the persistent status strip, so a learner cannot
  // see the run phase or terminal verdict while the results drawer is closed.
  it('shows a persistent status strip on tablet', () => {
    render(WorkspaceShell, {
      props: { problem, languages, layout: 'tablet' }
    });
    const strip = screen.getByRole('status', { name: 'Run status' });
    expect(strip.textContent).toContain('No run yet');
  });

  // Breaks if the narrow layout reorders regions away from the design's single column:
  // problem summary, then the editor and controls, then tests and results.
  it('stacks problem, editor and results in reading order when narrow', () => {
    render(WorkspaceShell, {
      props: { problem, languages, layout: 'narrow' }
    });
    const problemRegion = screen.getByRole('complementary', {
      name: 'Problem'
    });
    const editorRegion = screen.getByRole('region', { name: 'Editor' });
    const resultsRegion = screen.getByRole('region', { name: 'Results' });
    expect(
      problemRegion.compareDocumentPosition(editorRegion) &
        Node.DOCUMENT_POSITION_FOLLOWING
    ).toBeTruthy();
    expect(
      editorRegion.compareDocumentPosition(resultsRegion) &
        Node.DOCUMENT_POSITION_FOLLOWING
    ).toBeTruthy();
  });

  // Breaks if the ghost stream data the judge already sends never reaches the ghost
  // card: the learner would see verdicts but never the minimized counterexample.
  it('renders the ghost card when the live stream carries a ghost', async () => {
    const stream: RunStreamState = {
      ...initialRunState('run-ghost'),
      status: 'completed',
      ghost: ghostFixture()
    };
    render(WorkspaceShell, {
      props: {
        problem,
        languages,
        layout: 'desktop',
        sessionDeps: ghostSessionDeps(stream)
      }
    });

    const runButton = await screen.findByRole('button', {
      name: 'Run samples'
    });
    await fireEvent.click(runButton);

    const results = screen.getByRole('region', { name: 'Results' });
    await vi.waitFor(() => {
      expect(
        within(results).getByRole('region', { name: 'Ghost input' })
      ).toBeTruthy();
    });
    const ghostCard = within(results).getByRole('region', {
      name: 'Ghost input'
    });
    expect(
      within(ghostCard).getByText(/Minimized counterexample/)
    ).toBeTruthy();
    expect(within(ghostCard).getByText('Step 1: [3,5,0]')).toBeTruthy();
  });

  // Breaks if the Rerun ghost action posts to the public endpoint or drops the
  // stored ghost input, so the judge cannot rerun the exact failing case.
  it('reruns the stored ghost with the editor source through the ghost action', async () => {
    const ghostSend = vi.fn(() => Promise.resolve('run-ghost-2'));
    const ghostAction = vi.fn(() => ({ key: 'gk', send: ghostSend }));
    const stream: RunStreamState = {
      ...initialRunState('run-ghost'),
      status: 'completed',
      ghost: ghostFixture()
    };
    const sessionDeps = { ...ghostSessionDeps(stream), ghostAction };
    render(WorkspaceShell, {
      props: {
        problem,
        languages,
        layout: 'desktop',
        sessionDeps
      }
    });

    const runButton = await screen.findByRole('button', {
      name: 'Run samples'
    });
    await fireEvent.click(runButton);
    const rerun = await screen.findByRole('button', {
      name: /rerun ghost/i
    });
    await fireEvent.click(rerun);

    // test-change: GHO-3. The ghost-run endpoint requires ghost.submission_id (the
    // source run, which must belong to the caller), so the stored ghost is sent with
    // its run id. The old expectation sent the input alone, which the server refuses.
    expect(ghostAction).toHaveBeenCalledWith(
      expect.objectContaining({
        problem: 'two-sum',
        lang: 'python',
        ghost: { submission_id: 'run-ghost', input: [0, 0] }
      })
    );
  });

  // Breaks if "speech off" still sends the learner's code to the server for a bubble (R11.9).
  it('does not ask for the Familiar speech when the learner turned speech off', async () => {
    const requestSpeech = vi.fn(() =>
      Promise.resolve({
        kind: 'ready' as const,
        label: 'Measured summary' as const,
        text: 'Looks linear.'
      })
    );
    const loadPreferences = vi.fn(() =>
      Promise.resolve({ muted: false, hidden: false, speech: 'off' as const })
    );
    const savePreferences = vi.fn(() => Promise.resolve({ synced: true }));
    const stream: RunStreamState = {
      ...initialRunState('run-quiet'),
      status: 'completed',
      ghost: ghostFixture()
    };
    render(WorkspaceShell, {
      props: {
        problem,
        languages,
        layout: 'desktop',
        sessionDeps: ghostSessionDeps(stream),
        familiarDeps: { loadPreferences, savePreferences, requestSpeech }
      }
    });
    await vi.waitFor(() => expect(loadPreferences).toHaveBeenCalled());

    await fireEvent.click(
      await screen.findByRole('button', { name: 'Run samples' })
    );
    await screen.findByRole('button', { name: /rerun ghost/i });

    expect(requestSpeech).not.toHaveBeenCalled();
  });

  // Breaks if the speech request fires before the terminal read, or fires more
  // than once for the same run across republished stream states.
  it('asks for the Familiar speech once, after the run finishes', async () => {
    const requestSpeech = vi.fn(() =>
      Promise.resolve({
        kind: 'ready' as const,
        label: 'Measured summary' as const,
        text: 'Looks linear.'
      })
    );
    const loadPreferences = vi.fn(() =>
      Promise.resolve({ muted: false, hidden: false })
    );
    const savePreferences = vi.fn(() => Promise.resolve({ synced: true }));
    const stream: RunStreamState = {
      ...initialRunState('run-familiar'),
      status: 'completed',
      ghost: ghostFixture()
    };
    render(WorkspaceShell, {
      props: {
        problem,
        languages,
        layout: 'desktop',
        sessionDeps: ghostSessionDeps(stream),
        familiarDeps: { loadPreferences, savePreferences, requestSpeech }
      }
    });

    expect(loadPreferences).toHaveBeenCalledTimes(1);
    expect(requestSpeech).not.toHaveBeenCalled();

    const runButton = await screen.findByRole('button', {
      name: 'Run samples'
    });
    await fireEvent.click(runButton);

    await vi.waitFor(() => {
      expect(requestSpeech).toHaveBeenCalledWith('run-ghost');
    });
    expect(requestSpeech).toHaveBeenCalledTimes(1);
    expect(await screen.findByText(/Looks linear\./)).toBeTruthy();
  });

  // Breaks if the authored ladder is not reachable from the workspace, so a learner who fails
  // a run has no way to ask for the next rung (HNT-1: one level at a time, in order).
  it('reveals authored hints one rung at a time after a run finishes', async () => {
    const requestHint = vi.fn(async (_run: string, rung: number) => ({
      rung,
      text: `Authored rung ${rung}.`
    }));
    const stream: RunStreamState = {
      ...initialRunState('run-hints'),
      status: 'completed',
      ghost: ghostFixture()
    };
    render(WorkspaceShell, {
      props: {
        problem,
        languages,
        layout: 'desktop',
        sessionDeps: ghostSessionDeps(stream, 'run-hints'),
        familiarDeps: familiarOff(),
        requestHint
      }
    });
    expect(screen.queryByRole('button', { name: 'Show hint 1' })).toBeNull();

    await fireEvent.click(
      await screen.findByRole('button', { name: 'Run samples' })
    );
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Show hint 1' })
    );
    expect(await screen.findByText('Authored rung 1.')).toBeTruthy();
    expect(requestHint).toHaveBeenCalledWith('run-hints', 1);

    await fireEvent.click(
      await screen.findByRole('button', { name: 'Show hint 2' })
    );
    expect(await screen.findByText('Authored rung 2.')).toBeTruthy();
    expect(requestHint.mock.calls.map((call) => call[1])).toEqual([1, 2]);
  });

  // Breaks if a hint that answers late for an earlier run lands on the ladder of the next run,
  // or if a new run keeps the previous run's revealed hints (HNT-1: the ladder belongs to one run).
  it('starts a fresh ladder per run and drops a late reply for the old run', async () => {
    type Hint = { rung: number; text: string };
    const pending = new Map<string, (hint: Hint) => void>();
    const requestHint = vi.fn(
      (run: string) =>
        new Promise<Hint>((resolve) => {
          pending.set(run, resolve);
        })
    );
    const ids = ['run-a', 'run-b'];
    let started = 0;
    const deps: RunSessionDeps = {
      startAction: () => {
        const id = ids[Math.min(started, ids.length - 1)];
        started += 1;
        return { key: `key-${id}`, send: () => Promise.resolve(id) };
      },
      fetchRun: (id: string) =>
        Promise.resolve(
          parseRunView({
            run_id: id,
            problem: 'two-sum',
            lang: 'python',
            mode: 'run',
            status: 'completed',
            terminal: true,
            verdict: 'wrong_answer',
            attempt: 1,
            created_at: '2026-11-15T10:00:00Z',
            last_seq: 3,
            tests: []
          })
        ),
      watch: (options) => {
        options.onDone({
          ...initialRunState(ids[started - 1]),
          status: 'completed',
          ghost: ghostFixture()
        });
        return { close() {} };
      }
    };
    render(WorkspaceShell, {
      props: {
        problem,
        languages,
        layout: 'desktop',
        sessionDeps: deps,
        familiarDeps: familiarOff(),
        requestHint
      }
    });
    const run = await screen.findByRole('button', { name: 'Run samples' });
    await fireEvent.click(run);
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Show hint 1' })
    );
    expect(requestHint).toHaveBeenLastCalledWith('run-a', 1);

    // A second run finishes while run A's hint is still in flight.
    await fireEvent.click(run);
    await vi.waitFor(() => {
      expect(
        (
          screen.getByRole('button', {
            name: 'Show hint 1'
          }) as HTMLButtonElement
        ).disabled
      ).toBe(false);
    });
    // Run B asks for its own rung 1, then run A's old reply for rung 1 arrives first.
    await fireEvent.click(screen.getByRole('button', { name: 'Show hint 1' }));
    expect(requestHint).toHaveBeenLastCalledWith('run-b', 1);
    pending.get('run-a')?.({ rung: 1, text: 'Stale rung 1.' });
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(screen.queryByText('Stale rung 1.')).toBeNull();

    pending.get('run-b')?.({ rung: 1, text: 'Fresh rung 1.' });
    expect(await screen.findByText('Fresh rung 1.')).toBeTruthy();
  });

  // Breaks if a failed hint request shows invented text or drops the hints already revealed.
  it('keeps revealed hints and invents nothing when a hint request fails', async () => {
    const requestHint = vi
      .fn()
      .mockResolvedValueOnce({ rung: 1, text: 'Authored rung 1.' })
      .mockRejectedValueOnce(new Error('upstream secret detail'));
    const stream: RunStreamState = {
      ...initialRunState('run-hints-fail'),
      status: 'completed',
      ghost: ghostFixture()
    };
    render(WorkspaceShell, {
      props: {
        problem,
        languages,
        layout: 'desktop',
        sessionDeps: ghostSessionDeps(stream, 'run-hints-fail'),
        familiarDeps: familiarOff(),
        requestHint
      }
    });
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Run samples' })
    );
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Show hint 1' })
    );
    await screen.findByText('Authored rung 1.');
    await fireEvent.click(
      await screen.findByRole('button', { name: 'Show hint 2' })
    );
    expect(await screen.findByRole('alert')).toBeTruthy();
    expect(screen.getByText('Authored rung 1.')).toBeTruthy();
    expect(screen.queryByText(/Authored rung 2/)).toBeNull();
    expect(document.body.textContent).not.toContain('upstream secret');
    // The failed rung can be asked for again.
    expect(screen.getByRole('button', { name: 'Show hint 2' })).toBeTruthy();
  });

  // Breaks if leaving the page and coming back to the same finished run sends a second
  // speech request (FAM-6: at most one bubble per run, whatever the page does).
  it('sends one speech request for a run across a remount', async () => {
    const posts: string[] = [];
    vi.stubGlobal(
      'fetch',
      vi.fn(async (url: string, init?: RequestInit) => {
        if (init?.method === 'POST' && url.includes('/familiar-hint')) {
          posts.push(url);
          return new Response(
            JSON.stringify({
              source: 'template',
              body: 'Looks linear.',
              sent_to_provider: false
            }),
            { status: 200, headers: { 'content-type': 'application/json' } }
          );
        }
        // Preferences load and save: answer with a failure, which the page tolerates.
        return new Response('{}', { status: 500 });
      })
    );
    try {
      const stream: RunStreamState = {
        ...initialRunState('run-remount'),
        status: 'completed',
        ghost: ghostFixture()
      };
      for (let mount = 0; mount < 2; mount += 1) {
        render(WorkspaceShell, {
          props: {
            problem,
            languages,
            layout: 'desktop',
            sessionDeps: ghostSessionDeps(stream, 'run-remount')
          }
        });
        await fireEvent.click(
          await screen.findByRole('button', { name: 'Run samples' })
        );
        await vi.waitFor(() => {
          expect(screen.getByText(/Looks linear\./)).toBeTruthy();
        });
        cleanup();
      }
      expect(posts).toEqual(['/api/runs/run-remount/familiar-hint']);
    } finally {
      vi.unstubAllGlobals();
    }
  });

  // Breaks if a screen-reader user never hears the run finish, or if a stream
  // event moves focus away from wherever the learner was working.
  it('announces the terminal verdict politely and never steals focus', async () => {
    const stream: RunStreamState = {
      ...initialRunState('run-verdict'),
      status: 'completed',
      terminal: {
        kind: 'done',
        verdict: 'wrong_answer',
        passed: 0,
        total: 2,
        score: 0
      }
    };
    render(WorkspaceShell, {
      props: {
        problem,
        languages,
        layout: 'desktop',
        sessionDeps: ghostSessionDeps(stream)
      }
    });

    const runButton = await screen.findByRole('button', {
      name: 'Run samples'
    });
    runButton.focus();
    await fireEvent.click(runButton);

    const announcer = await screen.findByRole('status', {
      name: 'Run announcements'
    });
    await vi.waitFor(() => {
      expect(announcer.textContent).toContain('Wrong answer');
    });
    expect(document.activeElement).toBe(runButton);
  });

  // Breaks if leaving the page keeps a run watch or a pending request alive
  // behind the learner's back.
  it('closes the run watch and drops late results when the shell unmounts', async () => {
    const close = vi.fn();
    const fetchRun: RunSessionDeps['fetchRun'] = () => new Promise(() => {});
    const sessionDeps: RunSessionDeps = {
      startAction: () => ({
        key: 'k',
        send: () => Promise.resolve('run-late')
      }),
      fetchRun,
      watch: (options) => {
        options.onState(initialRunState('run-late'));
        return { close };
      }
    };
    const { unmount } = render(WorkspaceShell, {
      props: {
        problem,
        languages,
        layout: 'desktop',
        sessionDeps
      }
    });

    const runButton = await screen.findByRole('button', {
      name: 'Run samples'
    });
    await fireEvent.click(runButton);
    unmount();

    expect(close).toHaveBeenCalled();
  });
});
