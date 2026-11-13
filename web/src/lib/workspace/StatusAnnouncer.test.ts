// Tests for the polite status region used by the workspace (ide-workspace T10).
import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import StatusAnnouncer from './StatusAnnouncer.svelte';

afterEach(() => {
  cleanup();
});

describe('StatusAnnouncer', () => {
  // Breaks if status text is rendered as HTML, which would let program output or hint text run markup.
  it('shows the message as plain text, never as markup', () => {
    render(StatusAnnouncer, { props: { message: '<b>Accepted</b>' } });
    const status = screen.getByRole('status');
    expect(status.textContent).toBe('<b>Accepted</b>');
    expect(status.querySelector('b')).toBeNull();
  });
});
