// Tests for the dashboard practice calendar (costume-polish T04, CP-4).
import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import PracticeCalendar from './PracticeCalendar.svelte';

describe('PracticeCalendar', () => {
  afterEach(() => cleanup());

  it('renders activity and today in a readable five-week calendar', () => {
    render(PracticeCalendar, {
      props: {
        activityDays: ['2026-11-15', '2026-11-01'],
        today: '2026-11-15'
      }
    });

    expect(
      screen.getByRole('heading', { name: 'Practice calendar' })
    ).toBeTruthy();
    expect(
      screen.getByText(
        '2 practice days in the last 35 days. Most recent: 2026-11-15.'
      )
    ).toBeTruthy();
    expect(
      screen.getByLabelText('2026-11-15, practice day, today')
    ).toBeTruthy();
    expect(screen.getByLabelText('2026-11-01, practice day')).toBeTruthy();
    expect(screen.getByText('Today')).toBeTruthy();
    expect(screen.getByText('Practised')).toBeTruthy();
  });

  it('renders an understandable empty state without fabricating days', () => {
    render(PracticeCalendar, {
      props: { activityDays: [], today: '2026-11-15' }
    });

    expect(
      screen.getByText('No practice days in the last 35 days.')
    ).toBeTruthy();
    const activeCells = screen
      .getAllByRole('gridcell')
      .filter((cell) =>
        cell.getAttribute('aria-label')?.includes(', practice day')
      );
    expect(activeCells).toEqual([]);
  });
});
