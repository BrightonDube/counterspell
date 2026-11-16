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
  loadMonaco: () => new Promise(() => {})
}));

import ProblemWorkspace from './ProblemWorkspace.svelte';

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

beforeEach(() => {
  api.loadProblem.mockReset();
  api.loadLanguages.mockReset();
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
        screen.getByRole('heading', { level: 2, name: 'Two Sum' })
      ).toBeTruthy()
    );
    expect(screen.getByRole('region', { name: 'Editor' })).toBeTruthy();
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
