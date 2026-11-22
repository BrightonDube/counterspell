// Pure view model for the dashboard practice calendar (costume-polish T04).
// Dates are UTC `YYYY-MM-DD`; arithmetic uses Date.UTC so local time zones cannot shift a day.

export interface PracticeDay {
  date: string;
  label: string;
  active: boolean;
  today: boolean;
}

export interface PracticeCalendarView {
  weeks: PracticeDay[][];
  weekdays: string[];
  activeDays: string[];
  summary: string;
}

const WINDOW_DAYS = 35;
const WEEK_DAYS = 7;
const DAY_MS = 24 * 60 * 60 * 1000;
const WEEKDAY_NAMES = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

export function buildPracticeCalendar(
  activityDays: readonly string[],
  today: string
): PracticeCalendarView {
  const active = new Set(activityDays);
  const end = dayNumber(today);
  const start = end - (WINDOW_DAYS - 1);
  const days = Array.from({ length: WINDOW_DAYS }, (_, index) => {
    const date = formatDay(start + index);
    return {
      date,
      label: date.slice(8, 10),
      active: active.has(date),
      today: date === today
    };
  });
  const weeks = Array.from({ length: WINDOW_DAYS / WEEK_DAYS }, (_, index) =>
    days.slice(index * WEEK_DAYS, (index + 1) * WEEK_DAYS)
  );
  const weekdays = days
    .slice(0, WEEK_DAYS)
    .map((day) => WEEKDAY_NAMES[new Date(day.date + 'T00:00:00Z').getUTCDay()]);
  const activeDays = days
    .filter((day) => day.active)
    .map((day) => day.date)
    .reverse();
  const summary =
    activeDays.length === 0
      ? `No practice days in the last ${WINDOW_DAYS} days.`
      : `${activeDays.length} practice ${activeDays.length === 1 ? 'day' : 'days'} in the last ${WINDOW_DAYS} days. Most recent: ${activeDays[0]}.`;
  return { weeks, weekdays, activeDays, summary };
}

function dayNumber(date: string): number {
  const [year, month, day] = date.split('-').map(Number);
  return Math.floor(Date.UTC(year, month - 1, day) / DAY_MS);
}

function formatDay(day: number): string {
  const value = new Date(day * DAY_MS);
  const year = value.getUTCFullYear().toString().padStart(4, '0');
  const month = (value.getUTCMonth() + 1).toString().padStart(2, '0');
  const date = value.getUTCDate().toString().padStart(2, '0');
  return `${year}-${month}-${date}`;
}
