// Acceptance tests for output safety, accessibility and palette contrast
// (ide-workspace T23). Each test names the regression it catches.
import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { PublicTest, TestVerdict } from '../runs/types';
import type { GhostView } from '../ghost/types';
import Familiar from '../familiar/Familiar.svelte';
import { initialFamiliarState } from '../familiar/familiar-state';
import GhostPanel from './GhostPanel.svelte';
import TestPanel from './TestPanel.svelte';
import WorkspaceShell from './WorkspaceShell.svelte';
import { rowView } from './test-rows';
import type { LanguageSummary, ProblemDetail } from '../api/types';

afterEach(() => {
  cleanup();
});

const here = dirname(fileURLToPath(import.meta.url));
const appCss = readFileSync(resolve(here, '../../app.css'), 'utf8');

/** The workspace problem fixture, shared by the shell renders below. */
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
        { index: 1, input: [0, 0, 1], evaluations: 4 },
        { index: 2, input: [0, 0], evaluations: 9 }
      ],
      totalSteps: 3,
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

describe('output safety', () => {
  // Breaks if any workspace-side file renders untrusted data as HTML. The judge
  // sends text; the DOM must only ever hold it as text.
  it('uses no unsafe HTML sinks anywhere in workspace-facing source', () => {
    const sinks = [
      '{@html',
      'innerHTML',
      'outerHTML',
      'insertAdjacentHTML',
      'document.write',
      'eval('
    ];
    const files = import.meta.glob(
      [
        '../workspace/**/*.{svelte,ts}',
        '../ghost/**/*.ts',
        '../familiar/**/*.{svelte,ts}',
        '../runs/**/*.ts'
      ],
      { query: '?raw', import: 'default', eager: true }
    ) as Record<string, string>;
    for (const [path, source] of Object.entries(files)) {
      if (path.endsWith('.test.ts')) continue;
      for (const sink of sinks) {
        expect(source.includes(sink), `${path} contains ${sink}`).toBe(false);
      }
    }
  });

  // Breaks if a server message carrying markup reaches the page as markup.
  it('renders markup-looking output and ghost data as plain text', () => {
    const ghost = ghostFixture();
    ghost.explanation = '<img src=x onerror=alert(1)>';
    render(GhostPanel, {
      props: {
        status: 'completed',
        ghost,
        shrinkSteps: [],
        reducedMotion: false
      }
    });
    expect(screen.getByText('<img src=x onerror=alert(1)>')).toBeTruthy();
    expect(document.querySelector('img')).toBeNull();
  });
});

describe('status and hidden rows', () => {
  const verdicts: TestVerdict[] = [
    'pass',
    'wrong_answer',
    'runtime_error',
    'time_limit',
    'memory_limit',
    'output_limit'
  ];

  // Breaks if any verdict is shown by colour alone: every row must carry a
  // text label a reader who cannot see colour still receives.
  it('renders a text label for every verdict, not colour alone', () => {
    for (const verdict of verdicts) {
      const test: PublicTest = {
        sample: true,
        ordinal: 1,
        verdict,
        timeMs: 4,
        input: 'i',
        expected: 'e',
        got: 'g'
      };
      const row = rowView(test);
      expect(row.label.length, `${verdict} has no label`).toBeGreaterThan(0);
      expect(row.icon.length, `${verdict} has no icon`).toBeGreaterThan(0);
    }
  });

  // Breaks if a hidden test row leaks anything beyond ordinal, verdict and
  // time: the canary must never reach the page.
  it('builds a hidden row from ordinal, verdict and time only', () => {
    const hidden: PublicTest = {
      sample: false,
      ordinal: 7,
      verdict: 'wrong_answer',
      timeMs: 12
    };
    const row = rowView(hidden);
    const text = JSON.stringify(row);
    expect(text).not.toContain('input');
    expect(text).not.toContain('expected');
    expect(text).not.toContain('got');
    render(TestPanel, {
      props: { status: 'completed', tests: [hidden] }
    });
    const region = screen.getByRole('region', { name: 'Tests' });
    expect(region.textContent).not.toContain('input');
  });
});

describe('accessibility', () => {
  // Breaks if the shrink replay loses order or drops a recorded step.
  it('lists every recorded shrink step in recorded order', () => {
    render(GhostPanel, {
      props: {
        status: 'completed',
        ghost: ghostFixture(),
        shrinkSteps: [],
        reducedMotion: false
      }
    });
    const items = screen.getByRole('list').querySelectorAll('li');
    const texts = [...items].map((item) => item.textContent ?? '');
    expect(texts).toEqual([
      'Step 1: [3,5,0]',
      'Step 2: [0,0,1]',
      'Step 3: [0,0]'
    ]);
  });

  // Breaks if reduced motion removes information instead of motion.
  it('keeps every step under reduced motion, marked static', () => {
    render(GhostPanel, {
      props: {
        status: 'completed',
        ghost: ghostFixture(),
        shrinkSteps: [],
        reducedMotion: true
      }
    });
    const list = screen.getByRole('list');
    expect(list.querySelectorAll('li')).toHaveLength(3);
    expect(list.getAttribute('data-motion')).toBe('static');
  });

  // Breaks if the Familiar loses its polite live text.
  it('keeps a polite live region for the Familiar state', () => {
    render(Familiar, {
      props: { measured: initialFamiliarState() }
    });
    const live = document.querySelector('[aria-live="polite"]');
    expect(live).not.toBeNull();
    expect(live?.textContent).toContain('waiting');
  });

  // Breaks if any interactive element lacks a name or a 44px target.
  it('gives every shell control an accessible name and a 44px target', () => {
    render(WorkspaceShell, {
      props: { problem, languages, layout: 'desktop' }
    });
    for (const button of screen.getAllByRole('button')) {
      const name =
        button.getAttribute('aria-label') ?? button.textContent?.trim() ?? '';
      expect(name.length, 'a button has no accessible name').toBeGreaterThan(0);
      expect(
        /min-h-11|min-w-11|h-11|w-11/.test(button.className),
        `"${name}" is under the 44px target`
      ).toBe(true);
    }
    const select = screen.getByRole('combobox');
    expect(select.className).toContain('min-h-11');
  });
});

describe('palette contrast', () => {
  function luminance(hex: string): number {
    const channels = [1, 3, 5].map((i) => {
      const s = parseInt(hex.slice(i, i + 2), 16) / 255;
      return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
    });
    return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
  }

  function ratio(fg: string, bg: string): number {
    const a = luminance(fg);
    const b = luminance(bg);
    return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
  }

  function token(name: string): string {
    const match = appCss.match(
      // nosemgrep: javascript.lang.security.audit.detect-non-literal-regexp.detect-non-literal-regexp -- name is regex-escaped inline
      new RegExp(
        `--${name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}:\\s*(#[0-9a-fA-F]{6})`
      )
    );
    expect(match, `--${name} is missing from app.css`).not.toBeNull();
    const value = match?.[1];
    if (value === undefined) {
      throw new Error(`--${name} is missing from app.css`);
    }
    return value;
  }

  // Breaks if a palette change drops a text pair below WCAG AA (4.5:1).
  it('keeps every text pair at WCAG AA contrast or better', () => {
    const pairs: Array<[string, string]> = [
      ['text', 'bg'],
      ['text', 'surface'],
      ['text', 'raised'],
      ['text-muted', 'bg'],
      ['text-muted', 'surface'],
      ['text-subtle', 'surface'],
      ['accent-moss', 'surface'],
      ['accent-candle', 'bg'],
      ['accent-candle', 'surface'],
      ['info', 'surface'],
      ['success', 'surface'],
      ['success', 'success-bg'],
      ['danger', 'surface'],
      ['danger', 'danger-bg'],
      ['bg', 'accent-candle']
    ];
    for (const [fg, bg] of pairs) {
      const value = ratio(token(fg), token(bg));
      expect(
        value,
        `${fg} on ${bg} is ${value.toFixed(2)}:1`
      ).toBeGreaterThanOrEqual(4.5);
    }
  });

  // Breaks if the global focus ring disappears; every element relies on it.
  it('keeps a global focus-visible ring in the palette', () => {
    expect(appCss).toContain(':focus-visible');
    expect(appCss).toContain('outline: 2px solid var(--focus)');
  });
});
