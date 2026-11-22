// Tests for the dashboard practice calendar view model (costume-polish T04, CP-4).
import { describe, expect, it } from 'vitest';
import { buildPracticeCalendar } from './calendar';

describe('buildPracticeCalendar', () => {
  it('builds a five-week UTC window ending today', () => {
    const view = buildPracticeCalendar(
      ['2026-11-15', '2026-11-01'],
      '2026-11-15'
    );

    expect(view.weeks).toHaveLength(5);
    expect(view.weeks.flat()).toHaveLength(35);
    expect(view.weeks[0]?.[0]?.date).toBe('2026-10-12');
    expect(view.weeks[4]?.[6]?.date).toBe('2026-11-15');
    expect(view.weeks[4]?.[6]).toMatchObject({
      active: true,
      today: true,
      label: '15'
    });
    expect(view.weeks[2]?.[6]?.date).toBe('2026-11-01');
    expect(view.weeks[2]?.[6]?.active).toBe(true);
    expect(view.weekdays).toEqual([
      'Mon',
      'Tue',
      'Wed',
      'Thu',
      'Fri',
      'Sat',
      'Sun'
    ]);
  });

  it('aligns weekday headers to the first day of a non-Monday window', () => {
    const view = buildPracticeCalendar([], '2026-11-12');

    expect(view.weeks[0]?.[0]?.date).toBe('2026-10-09');
    expect(view.weekdays).toEqual([
      'Fri',
      'Sat',
      'Sun',
      'Mon',
      'Tue',
      'Wed',
      'Thu'
    ]);
  });

  it('ignores activity outside the displayed window in the text summary', () => {
    const view = buildPracticeCalendar(
      ['2026-10-11', '2026-11-15'],
      '2026-11-15'
    );

    expect(view.activeDays).toEqual(['2026-11-15']);
    expect(view.summary).toBe(
      '1 practice day in the last 35 days. Most recent: 2026-11-15.'
    );
  });

  it('crosses a month boundary in UTC without using local time', () => {
    const view = buildPracticeCalendar(['2026-10-31'], '2026-11-01');

    expect(view.weeks[4]?.[6]?.date).toBe('2026-11-01');
    expect(view.weeks[4]?.[5]).toMatchObject({
      date: '2026-10-31',
      active: true,
      today: false
    });
  });

  it('keeps active dates and reports an empty window without inventing activity', () => {
    const view = buildPracticeCalendar([], '2026-11-15');

    expect(view.activeDays).toEqual([]);
    expect(view.summary).toBe('No practice days in the last 35 days.');
    expect(view.weeks.flat().every((day) => !day.active)).toBe(true);
  });

  it('summarises active days newest first', () => {
    const view = buildPracticeCalendar(
      ['2026-11-01', '2026-11-15'],
      '2026-11-15'
    );

    expect(view.activeDays).toEqual(['2026-11-15', '2026-11-01']);
    expect(view.summary).toBe(
      '2 practice days in the last 35 days. Most recent: 2026-11-15.'
    );
  });
});
