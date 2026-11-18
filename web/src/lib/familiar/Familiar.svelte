<script lang="ts">
  import type { SpeechResult } from '../api/familiar';
  import { familiarView, type FamiliarState } from './familiar-state';
  import Mascot from './mascot/Mascot.svelte';

  let {
    measured,
    reducedMotion = false,
    prefs,
    speech = null,
    onPrefsChange
  }: {
    measured: FamiliarState;
    reducedMotion?: boolean;
    /** Server-loaded choices. Local toggles mirror them and notify the page, which persists them. */
    prefs?: { muted: boolean; hidden: boolean };
    /** The speech bubble for the finished run, or null while none exists. */
    speech?: SpeechResult | null;
    onPrefsChange?: (next: { muted: boolean; hidden: boolean }) => void;
  } = $props();

  // Local toggles matter only when no server prefs are supplied; once `prefs`
  // exists it is the source of truth and the page persists every change.
  let localMuted = $state(false);
  let localHidden = $state(false);
  const muted = $derived(prefs?.muted ?? localMuted);
  const hidden = $derived(prefs?.hidden ?? localHidden);

  const view = $derived(
    familiarView(measured, { muted, hidden }, reducedMotion)
  );

  function toggleMuted(): void {
    localMuted = !muted;
    onPrefsChange?.({ muted: localMuted, hidden });
  }

  function toggleHidden(): void {
    localHidden = !hidden;
    onPrefsChange?.({ muted, hidden: localHidden });
  }

  function unhide(): void {
    localHidden = false;
    onPrefsChange?.({ muted, hidden: false });
  }

  // A failed run uses the danger text colour. A live result plays its burst only once.
  const failed = $derived(
    measured.terminal !== null && measured.terminal.verdict !== 'accepted'
  );
  const effect = $derived(
    !measured.live
      ? 'none'
      : failed && measured.mood === 1
        ? 'crack_burst'
        : measured.mood === 5
          ? 'laugh_burst'
          : 'none'
  );
</script>

<!-- Only measured facts reach this component. The text below is the full equivalent of the figure. -->
<section
  aria-labelledby="familiar-heading"
  class="flex flex-col items-center gap-3 rounded-lg border border-border bg-surface p-4"
>
  <div class="flex w-full items-center justify-between gap-2">
    <h2 id="familiar-heading" class="text-base font-semibold text-text">
      Efficiency Familiar
    </h2>
    <div class="flex gap-2">
      <button
        type="button"
        aria-pressed={muted}
        class="min-h-11 rounded border border-border-strong px-3 text-sm text-text"
        onclick={toggleMuted}
      >
        {muted ? 'Unmute' : 'Mute'}
      </button>
      <button
        type="button"
        aria-pressed={hidden}
        class="min-h-11 rounded border border-border-strong px-3 text-sm text-text"
        onclick={toggleHidden}
      >
        {hidden ? 'Show' : 'Hide'}
      </button>
    </div>
  </div>

  {#if view.mood !== null}
    <Mascot
      mood={view.mood}
      reason="measured"
      headline={view.headline}
      badge={view.badge}
      {failed}
      {effect}
      live={measured.live}
      {muted}
      {hidden}
      {reducedMotion}
      onToggleHidden={unhide}
    />
  {:else}
    <p aria-live="polite" class="text-sm text-text-muted">{view.liveText}</p>
  {/if}

  {#if speech !== null}
    <p role="status" class="w-full rounded bg-bg px-3 py-2 text-sm text-text">
      {#if speech.kind === 'ready'}
        <span class="font-semibold">{speech.label}:</span>
      {/if}
      {speech.text}
    </p>
  {/if}

  <details class="w-full text-sm text-text-muted">
    <summary class="cursor-pointer text-text">How does it judge me?</summary>
    <ul class="mt-2 list-disc space-y-1 pl-5">
      <li>Times use medians of repeated runs, not single measurements.</li>
      <li>
        Growth is read from size ratios, and the fit must agree before a class
        is reported.
      </li>
      <li>
        Confidence is shown. A low-confidence reading is only a rough guide.
      </li>
      <li>Constant factors are compared only with a same-language baseline.</li>
      <li>A shared server adds noise, so small inputs can mislead.</li>
      <li>
        Some problems have no meaningful growth curve. Those show only pass or
        fail.
      </li>
    </ul>
  </details>
</section>
