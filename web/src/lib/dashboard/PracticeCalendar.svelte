<script lang="ts">
  // Read-only UTC practice calendar. The text summary and hidden active-day list carry the same facts as colour.
  import { buildPracticeCalendar } from './calendar';

  let {
    activityDays,
    today = new Date().toISOString().slice(0, 10)
  }: { activityDays: readonly string[]; today?: string } = $props();

  const calendar = $derived(buildPracticeCalendar(activityDays, today));
</script>

<div class="space-y-4">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <h2 id="calendar-title" class="text-lg font-semibold">Practice calendar</h2>
    <div class="flex items-center gap-4 text-xs text-text-muted">
      <span class="inline-flex items-center gap-1.5">
        <span aria-hidden="true" class="h-2.5 w-2.5 rounded-full bg-accent-moss"
        ></span>
        Practised
      </span>
      <span class="inline-flex items-center gap-1.5">
        <span
          aria-hidden="true"
          class="h-2.5 w-2.5 rounded-full border border-accent-candle"
        ></span>
        Today
      </span>
    </div>
  </div>

  <div class="space-y-2" role="grid" aria-label="Last 35 days">
    <div
      class="grid grid-cols-7 gap-1 text-center text-[0.65rem] font-medium uppercase tracking-wide text-text-muted"
    >
      {#each calendar.weekdays as weekday (weekday)}
        <span aria-hidden="true">{weekday}</span>
      {/each}
    </div>
    {#each calendar.weeks as week, weekIndex (weekIndex)}
      <div class="grid grid-cols-7 gap-1" role="row">
        {#each week as day (day.date)}
          <div
            role="gridcell"
            aria-label={`${day.date}${day.active ? ', practice day' : ', no practice'}${day.today ? ', today' : ''}`}
            class={`flex min-h-9 items-center justify-center rounded-md border text-xs tabular-nums transition-colors duration-150 ${
              day.active
                ? 'border-accent-moss/50 bg-accent-moss/20 font-semibold text-accent-moss'
                : 'border-border bg-raised/70 text-text-muted'
            } ${
              day.today
                ? 'ring-2 ring-accent-candle ring-offset-1 ring-offset-surface forced-colors:underline forced-colors:decoration-2'
                : ''
            }`}
          >
            {day.label}
          </div>
        {/each}
      </div>
    {/each}
  </div>

  <p class="text-sm text-text-muted">{calendar.summary}</p>
  <ul class="sr-only" aria-label="Active practice days">
    {#each calendar.activeDays as day (day)}
      <li>{day}</li>
    {/each}
  </ul>
</div>
