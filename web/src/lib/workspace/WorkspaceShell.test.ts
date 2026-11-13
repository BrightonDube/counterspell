// Component tests for the responsive workspace shell (ide-workspace T09). Each test names the layout break it catches.
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { LanguageSummary, ProblemDetail } from '../api/types';

vi.mock('../editor/monaco-loader', () => ({
  CRYPT_THEME_NAME: 'crypt',
  loadMonaco: () => new Promise(() => {})
}));

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
    render(WorkspaceShell, { props: { problem, languages } });
    expect(screen.getByRole('complementary', { name: 'Problem' })).toBeTruthy();
    expect(screen.getByRole('region', { name: 'Editor' })).toBeTruthy();
    expect(screen.getByRole('region', { name: 'Results' })).toBeTruthy();
  });

  // Breaks if the problem column cannot be collapsed, which leaves no room for the editor on a tablet.
  it('collapses and restores the problem column from its toggle', async () => {
    render(WorkspaceShell, { props: { problem, languages } });
    const toggle = screen.getByRole('button', { name: 'Hide problem' });
    const aside = screen.getByRole('complementary', { name: 'Problem' });
    expect(toggle.getAttribute('aria-expanded')).toBe('true');

    await fireEvent.click(toggle);
    expect(toggle.getAttribute('aria-expanded')).toBe('false');
    expect(aside.hidden).toBe(true);
    expect(screen.getByRole('button', { name: 'Show problem' })).toBeTruthy();
  });
});
