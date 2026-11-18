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
      seed: 7,
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
function ghostSessionDeps(stream: RunStreamState): RunSessionDeps {
  return {
    startAction: () => ({
      key: 'ghost-key',
      send: () => Promise.resolve('run-ghost')
    }),
    fetchRun: () =>
      Promise.resolve(
        parseRunView({
          run_id: 'run-ghost',
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

const problem: ProblemDetail = {
  slug: 'two-sum',
  title: 'Two Sum',
  track: { slug: 'arrays', name: 'Crypt of Arrays' },
  concepts: [],
  difficulty: 'intro',
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
    enabled: true,
    fileName: 'solution.py',
    maxSourceBytes: 65536,
    supportsRun: true,
    supportsSubmit: true
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

    expect(ghostAction).toHaveBeenCalledWith(
      expect.objectContaining({
        problem: 'two-sum',
        lang: 'python',
        ghost: { input: [0, 0] }
      })
    );
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
    expect(
      await screen.findByText(/Looks linear\./)
    ).toBeTruthy();
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
});
