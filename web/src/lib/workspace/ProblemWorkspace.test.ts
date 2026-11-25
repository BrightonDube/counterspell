// Tests for the workspace loader: it fetches the problem and languages, then shows the shell or an honest error.
import { cleanup, render, screen, waitFor } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiRequestError } from '../api/client';
import type { LanguageSummary, ProblemDetail } from '../api/types';

const api = vi.hoisted(() => ({
  loadProblem: vi.fn(),
  loadLanguages: vi.fn()
}));

vi.mock('../api/problems', () => ({ loadProblem: api.loadProblem }));
vi.mock('../api/languages', () => ({ loadLanguages: api.loadLanguages }));
vi.mock('../editor/monaco-loader', () => ({
  CRYPT_THEME_NAME: 'crypt',
  loadMonaco: () => new Promise(() => {}),
  loadMonacoLanguage: () => Promise.resolve()
}));

import ProblemWorkspace from './ProblemWorkspace.svelte';

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

beforeEach(() => {
  api.loadProblem.mockReset();
  api.loadLanguages.mockReset();
  // happy-dom defaults to a 1024px viewport, which resolves to the shell's tablet
  // mode; these tests assert the desktop shell, so emulate a wide viewport.
  (
    window as unknown as {
      happyDOM?: { setViewport(v: { width: number; height: number }): void };
    }
  ).happyDOM?.setViewport({ width: 1440, height: 900 });
});

afterEach(() => {
  cleanup();
});

describe('ProblemWorkspace', () => {
  // Breaks if the shell appears before the problem is loaded, which would show an empty statement as if it were real.
  it('shows the workspace once the problem and languages have loaded', async () => {
    api.loadProblem.mockResolvedValue(problem);
    api.loadLanguages.mockResolvedValue(languages);
    render(ProblemWorkspace, { props: { slug: 'two-sum' } });

    await waitFor(() =>
      expect(
        screen.getByRole('heading', { level: 1, name: 'Two Sum' })
      ).toBeTruthy()
    );
    expect(screen.getByRole('region', { name: 'Editor' })).toBeTruthy();
  });

  // Breaks if the browser performance check cannot see when the problem detail arrived (R15.5).
  it('marks problem-detail-loaded once the problem is shown', async () => {
    const mark = vi.spyOn(performance, 'mark');
    try {
      api.loadProblem.mockResolvedValue(problem);
      api.loadLanguages.mockResolvedValue(languages);
      render(ProblemWorkspace, { props: { slug: 'two-sum' } });
      await waitFor(() =>
        expect(mark.mock.calls).toContainEqual(['problem-detail-loaded'])
      );
      expect(JSON.stringify(mark.mock.calls)).not.toContain('two-sum');
    } finally {
      mark.mockRestore();
    }
  });

  // Breaks if the picker offers a language the problem has no starter for — choosing it would
  // open a blank editor, exactly the LC01 mismatch this spec fixes.
  it('offers only the languages the problem enables', async () => {
    api.loadProblem.mockResolvedValue(problem);
    api.loadLanguages.mockResolvedValue([
      ...languages,
      { id: 'java', displayName: 'Java', maxSourceBytes: 65536 },
      { id: 'cpp', displayName: 'C++', maxSourceBytes: 65536 }
    ]);
    render(ProblemWorkspace, { props: { slug: 'two-sum' } });

    const select = await waitFor(() =>
      screen.getByRole('combobox', { name: 'Language' })
    );
    const options = Array.from(select.querySelectorAll('option')).map(
      (option) => option.textContent
    );
    expect(options).toEqual(['Python']);
    expect(screen.queryByRole('option', { name: 'Java' })).toBeNull();
  });

  // Breaks if the problem route loses its one descriptive h1 (AUD-13) — the
  // whole app reads this heading, and it is what search results should show.
  it('renders the problem title as the page h1', async () => {
    api.loadProblem.mockResolvedValue(problem);
    api.loadLanguages.mockResolvedValue(languages);
    render(ProblemWorkspace, { props: { slug: 'two-sum' } });

    await waitFor(() =>
      expect(
        screen.getByRole('heading', { level: 1, name: 'Two Sum' })
      ).toBeTruthy()
    );
    expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1);
  });

  // Breaks if the route stops handing the loaded problem upward — the page needs it
  // to write the real title and description into the head (AQ01/AUD-12).
  it('reports the loaded problem to the page for route metadata', async () => {
    api.loadProblem.mockResolvedValue(problem);
    api.loadLanguages.mockResolvedValue(languages);
    const onready = vi.fn();
    render(ProblemWorkspace, { props: { slug: 'two-sum', onready } });

    await waitFor(() => expect(onready).toHaveBeenCalledWith(problem));
  });

  // Breaks if a missing problem shows a blank page or a raw error instead of the server's safe message.
  it('shows the safe message for a problem that does not exist', async () => {
    api.loadProblem.mockRejectedValue(
      new ApiRequestError(404, 'not_found', 'That problem does not exist.')
    );
    api.loadLanguages.mockResolvedValue(languages);
    render(ProblemWorkspace, { props: { slug: 'missing' } });

    await waitFor(() =>
      expect(screen.getByRole('alert').textContent).toContain(
        'That problem does not exist.'
      )
    );
    expect(screen.queryByRole('region', { name: 'Editor' })).toBeNull();
  });
});
