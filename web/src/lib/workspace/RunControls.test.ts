// Component tests for the Run and Submit controls (ide-workspace T10). Each test names the break it catches.
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import RunControls from './RunControls.svelte';

afterEach(() => {
  cleanup();
});

describe('RunControls', () => {
  // Breaks if a double click starts two runs for one press, which would also burn the rate limit.
  it('ignores clicks while a run is already in flight', async () => {
    let runs = 0;
    render(RunControls, {
      props: {
        busy: true,
        sourceBytes: 10,
        limitBytes: 65536,
        onRun: () => (runs += 1),
        onSubmit: () => {}
      }
    });
    const run = screen.getByRole('button', {
      name: /run samples/i
    }) as HTMLButtonElement;
    expect(run.disabled).toBe(true);
    await fireEvent.click(run);
    expect(runs).toBe(0);
  });

  // Breaks if an oversized source shows no warning, so the learner only finds out from the server.
  it('warns with the byte count when the source is too large', () => {
    render(RunControls, {
      props: {
        busy: false,
        sourceBytes: 70000,
        limitBytes: 65536,
        onRun: () => {},
        onSubmit: () => {}
      }
    });
    expect(screen.getByRole('alert').textContent).toContain('70000 bytes');
  });

  // Breaks if a too-large source is still sent, so the warning does not stop the request.
  it('disables Run and Submit when the source is over the byte limit', () => {
    render(RunControls, {
      props: {
        busy: false,
        sourceBytes: 70000,
        limitBytes: 65536,
        onRun: () => {},
        onSubmit: () => {}
      }
    });
    expect(
      (
        screen.getByRole('button', {
          name: /run samples/i
        }) as HTMLButtonElement
      ).disabled
    ).toBe(true);
    expect(
      (screen.getByRole('button', { name: /submit/i }) as HTMLButtonElement)
        .disabled
    ).toBe(true);
  });

  // Breaks if Ctrl+Enter stops running samples, so keyboard users lose the fast path.
  it('runs samples on Ctrl+Enter and submits on Ctrl+Shift+Enter', async () => {
    const calls: string[] = [];
    render(RunControls, {
      props: {
        busy: false,
        sourceBytes: 10,
        limitBytes: 65536,
        onRun: () => calls.push('run'),
        onSubmit: () => calls.push('submit')
      }
    });
    await fireEvent.keyDown(window, { key: 'Enter', ctrlKey: true });
    await fireEvent.keyDown(window, {
      key: 'Enter',
      ctrlKey: true,
      shiftKey: true
    });
    expect(calls).toEqual(['run', 'submit']);
  });

  // Breaks if a held key repeats a run while one is already in flight.
  it('ignores the shortcuts while a run is in flight', async () => {
    let runs = 0;
    render(RunControls, {
      props: {
        busy: true,
        sourceBytes: 10,
        limitBytes: 65536,
        onRun: () => (runs += 1),
        onSubmit: () => {}
      }
    });
    await fireEvent.keyDown(window, { key: 'Enter', ctrlKey: true });
    expect(runs).toBe(0);
  });

  // Breaks if the shortcut fires while the learner types in a form field outside the editor.
  it('leaves Enter alone inside a text field outside the editor', async () => {
    let runs = 0;
    render(RunControls, {
      props: {
        busy: false,
        sourceBytes: 10,
        limitBytes: 65536,
        onRun: () => (runs += 1),
        onSubmit: () => {}
      }
    });
    const field = document.createElement('textarea');
    document.body.appendChild(field);
    await fireEvent.keyDown(field, { key: 'Enter', ctrlKey: true });
    field.remove();
    expect(runs).toBe(0);
  });

  // Breaks if the shortcut help disappears, so keyboard users cannot discover the shortcuts.
  it('lists both shortcuts as plain text', () => {
    render(RunControls, {
      props: {
        busy: false,
        sourceBytes: 10,
        limitBytes: 65536,
        onRun: () => {},
        onSubmit: () => {}
      }
    });
    expect(
      screen.getByText(
        'Ctrl+Enter or Cmd+Enter runs samples. Ctrl+Shift+Enter or Cmd+Shift+Enter submits.'
      )
    ).toBeTruthy();
  });

  // Breaks if the save state is not announced to screen readers.
  it('announces the save message in a polite status region', () => {
    render(RunControls, {
      props: {
        busy: false,
        sourceBytes: 10,
        limitBytes: 65536,
        saveMessage: 'Draft saved',
        onRun: () => {},
        onSubmit: () => {}
      }
    });
    const status = screen.getByRole('status');
    expect(status.textContent).toBe('Draft saved');
    expect(status.getAttribute('aria-live')).toBe('polite');
  });
});
