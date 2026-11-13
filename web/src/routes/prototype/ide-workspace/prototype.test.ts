// Tests for the fixture-only prototype route (ide-workspace T04). Each test names the break it catches.
import { cleanup, render, screen, fireEvent } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import Page from './+page.svelte';
import {
  isFixtureRunId,
  PROTOTYPE_BANNER
} from '../../../lib/prototype/fixtures';

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe('prototype route', () => {
  // Breaks if the visual fixture can be mistaken for a real judge result, which the spec forbids.
  it('shows the persistent fixture banner', () => {
    render(Page);
    expect(screen.getByText(PROTOTYPE_BANNER)).toBeTruthy();
  });

  // Breaks if a fixture action reaches a production run endpoint and starts a real judge run.
  it('never calls fetch when a fixture action is pressed', async () => {
    const fetchSpy = vi.spyOn(globalThis, 'fetch');
    render(Page);
    await fireEvent.click(
      screen.getByRole('button', { name: /run samples \(fixture\)/i })
    );
    expect(fetchSpy).not.toHaveBeenCalled();
    expect(screen.getByText(/no judge run was started/i)).toBeTruthy();
  });
});

describe('fixture identifiers', () => {
  // Breaks if a fixture run id looks like a real run id and could be sent to a production endpoint.
  it('recognises only the fixture prefix as a fixture run id', () => {
    expect(isFixtureRunId('fixture-run-0001')).toBe(true);
    expect(isFixtureRunId('3f2c9a1e-7b44-4c1d-9e2a-0d5f6a7b8c9d')).toBe(false);
  });
});
