// Component tests for the editor panel (ide-workspace T09). Monaco is kept from loading so the panel is tested alone.
import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { LanguageSummary, ProblemDetail } from '../api/types';

vi.mock('../editor/monaco-loader', () => ({
  CRYPT_THEME_NAME: 'crypt',
  loadMonaco: () => new Promise(() => {})
}));

import EditorPanel from './EditorPanel.svelte';

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
  starters: { python: 'def two_sum():\n    pass\n' },
  enabledLanguages: ['python']
};

function language(
  id: string,
  displayName: string,
  enabled: boolean
): LanguageSummary {
  return {
    id,
    displayName,
    enabled,
    fileName: `solution.${id === 'python' ? 'py' : 'js'}`,
    maxSourceBytes: 65536,
    supportsRun: enabled,
    supportsSubmit: enabled
  };
}

describe('EditorPanel', () => {
  // Breaks if a disabled language is offered, so a learner picks a language the server then refuses.
  it('offers only the languages the server enables', () => {
    render(EditorPanel, {
      props: {
        problem,
        languages: [
          language('python', 'Python', true),
          language('javascript', 'JavaScript', false)
        ]
      }
    });
    const options = screen
      .getAllByRole('option')
      .map((option) => option.textContent);
    expect(options).toEqual(['Python']);
  });

  // Breaks if the panel shows a save message before anything was edited, which would look like a saved draft.
  it('shows no draft status before the learner edits', () => {
    const { container } = render(EditorPanel, {
      props: { problem, languages: [language('python', 'Python', true)] }
    });
    const status = container.querySelector('p[aria-live="polite"]');
    expect(status?.textContent?.trim()).toBe('');
  });
});
