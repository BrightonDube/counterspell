// Component tests for the workspace console (costume-polish T06). Each test names the
// disclosure or output-safety break it catches.
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import ConsolePanel from './ConsolePanel.svelte';
import type { ConsoleEntry } from './console-view';

afterEach(() => cleanup());

const entries: ConsoleEntry[] = [
  { id: 'compile', heading: 'Compile output', text: 'warning' },
  {
    id: 'sample-1-output',
    heading: 'Sample 1 output',
    text: '<b>raw text</b>'
  },
  { id: 'sample-1-stderr', heading: 'Sample 1 stderr', text: 'diagnostic' }
];

describe('ConsolePanel', () => {
  it('starts as an accessible collapsed disclosure with a text status', () => {
    render(ConsolePanel, {
      props: { entries: [], open: false, onToggle: () => {} }
    });

    const region = screen.getByRole('region', { name: 'Console' });
    const button = screen.getByRole('button', { name: 'Console' });
    expect(region).toBeTruthy();
    expect(button.getAttribute('aria-expanded')).toBe('false');
    expect(button.getAttribute('aria-controls')).toBe(
      'workspace-console-output'
    );
    expect(screen.getByText('No console output yet.')).toBeTruthy();
  });

  it('reports the toggle and exposes labelled plain-text output when open', async () => {
    const onToggle = vi.fn();
    const { container } = render(ConsolePanel, {
      props: { entries, open: true, onToggle }
    });

    await fireEvent.click(screen.getByRole('button', { name: 'Console' }));
    expect(onToggle).toHaveBeenCalledTimes(1);
    expect(screen.getByText('Compile output')).toBeTruthy();
    expect(screen.getByText('Sample 1 output')).toBeTruthy();
    expect(screen.getByText('Sample 1 stderr')).toBeTruthy();
    expect(screen.getByText('<b>raw text</b>')).toBeTruthy();
    expect(container.querySelector('b')).toBeNull();
  });
});
