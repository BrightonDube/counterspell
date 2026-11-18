// Component tests for the responsive workspace shell (ide-workspace T09). Each test names the layout break it catches.
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
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
import WorkspaceShell from './WorkspaceShell.svelte';

afterEach(() => {
  cleanup();
});

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
});
