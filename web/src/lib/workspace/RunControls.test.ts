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
});
