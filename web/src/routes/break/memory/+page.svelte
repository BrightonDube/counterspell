<script>
  import AppShell from '../../../lib/shell/AppShell.svelte';
  import { resolve } from '$app/paths';
  import { onMount } from 'svelte';
  import { TILE_ART, checkPress, level, newRound } from '#lib/break/memory.js';
  import { saveScore } from '#lib/break/api.js';
  import { session } from '#lib/auth/session.svelte.js';
  import Seo from '../../../lib/seo/Seo.svelte';

  const TILES = 9;
  /** @type {number[]} */
  let sequence = $state([]);
  /** @type {number[]} */
  let presses = $state([]);
  let lit = $state(-1);
  let revealTile = $state(-1);
  /** @type {'idle' | 'showing' | 'input' | 'over'} */
  let phase = $state('idle');
  let announce = $state('Press start to play Lantern recall.');
  let flashTile = $state(-1);
  /** @type {'' | 'correct' | 'wrong'} */
  let flashKind = $state('');
  // Each accepted press owns its own verdict/cleanup pair — a later press
  // cancels the earlier ones so its reveal gets the full window.
  /** @type {ReturnType<typeof setTimeout> | undefined} */
  let flashTimer;
  /** @type {ReturnType<typeof setTimeout> | undefined} */
  let clearTimer;
  let focusIdx = $state(0);
  let signedOut = $state(false);
  let saveError = $state('');
  /** @type {HTMLButtonElement[]} */
  let tileEls = [];
  let reduced = false;

  async function showSequence() {
    phase = 'showing';
    if (reduced) {
      announce =
        'Sequence: ' + sequence.map((t) => `tile ${t + 1}`).join(', ') + '.';
      await new Promise((r) => setTimeout(r, 300));
    } else {
      for (const t of sequence) {
        lit = t;
        announce = `Tile ${t + 1}`;
        await new Promise((r) => setTimeout(r, 600));
        lit = -1;
        await new Promise((r) => setTimeout(r, 200));
      }
    }
    presses = [];
    phase = 'input';
    announce = 'Your turn. Repeat the sequence.';
  }

  async function start() {
    sequence = newRound(newRound(newRound([])));
    presses = [];
    await showSequence();
  }

  /** @param {number} i */
  function press(i) {
    if (phase !== 'input') return;
    presses = presses.concat(i);
    const verdict = checkPress(sequence, presses, presses.length - 1);
    // The themed image shows first; the verdict colour washes over it a beat
    // later so the player sees what the tile hid before the judgement lands.
    revealTile = i;
    const kind = verdict === 'wrong' ? 'wrong' : 'correct';
    clearTimeout(flashTimer);
    clearTimeout(clearTimer);
    flashTile = -1;
    if (verdict === 'wrong') {
      phase = 'over';
      const lvl = level(sequence);
      announce = `Wrong, game over at level ${lvl}.`;
      void finalizeScore(lvl);
    } else {
      announce = verdict === 'round_done' ? 'Correct. Next round.' : 'Correct.';
      if (verdict === 'round_done') {
        sequence = newRound(sequence);
        setTimeout(showSequence, 800);
      }
    }
    flashTimer = setTimeout(() => {
      flashTile = i;
      flashKind = kind;
    }, 180);
    clearTimer = setTimeout(() => {
      revealTile = -1;
      flashTile = -1;
    }, 560);
  }

  // Await the session answer before deciding — a probe still in flight must not
  // drop a signed-in player's score, and an anonymous player must never trigger
  // the scores endpoint's expected 401 (AUD-19/AUD-20).
  /** @param {number} lvl */
  function finalizeScore(lvl) {
    void session
      .load()
      .then(() => {
        if (session.status === 'signed_in') {
          void saveScore({ game: 'brain', level: lvl });
        } else {
          signedOut = true;
        }
      })
      .catch(() => {
        saveError = 'Could not check your session — the score was not saved.';
      });
  }

  /** @param {KeyboardEvent} event @param {number} i */
  function onKey(event, i) {
    const move = /** @type {Record<string, number>} */ ({
      ArrowRight: 1,
      ArrowLeft: -1,
      ArrowDown: 3,
      ArrowUp: -3
    })[event.key];
    if (move !== undefined) {
      event.preventDefault();
      focusIdx = (focusIdx + move + TILES) % TILES;
      tileEls[focusIdx]?.focus();
    } else if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      press(i);
    }
  }

  onMount(() => {
    reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
  });
</script>

<Seo
  title="Lantern recall · Counterspell"
  description="Repeat the growing sequence of nine lanterns — a memory game in the break room."
  path="/break/memory"
/>

<AppShell>
  <div class="mx-auto max-w-md space-y-4">
    <h1 class="text-2xl font-bold">Lantern recall</h1>
    <p class="text-sm text-text-muted">
      Watch the tiles light up, then repeat the order.
    </p>
    <div aria-live="polite" class="min-h-6 text-sm">{announce}</div>
    {#if signedOut}
      <p class="text-sm text-text-muted">
        <a
          class="text-accent underline"
          href={resolve('/login?return_to=/break/memory')}
        >
          Sign in
        </a>
        to save your score.
      </p>
    {/if}
    {#if saveError}
      <p role="alert" class="text-sm text-danger">{saveError}</p>
    {/if}
    <div class="grid grid-cols-3 gap-2" role="group" aria-label="Memory tiles">
      {#each Array(TILES) as _, i (i)}
        <button
          bind:this={tileEls[i]}
          type="button"
          tabindex={focusIdx === i ? 0 : -1}
          onfocus={() => (focusIdx = i)}
          onclick={() => press(i)}
          onkeydown={(e) => onKey(e, i)}
          class="relative aspect-square overflow-hidden rounded-lg border text-sm font-medium transition-colors
          {lit === i || revealTile === i ? 'tile-reveal' : ''}
          {flashTile === i && flashKind === 'correct' ? 'tile-correct' : ''}
          {flashTile === i && flashKind === 'wrong' ? 'tile-wrong' : ''}
          {phase === 'input'
            ? 'border-border bg-surface'
            : 'border-border bg-raised'}"
          aria-label={'Tile ' + (i + 1)}
        >
          <img
            class="tile-art"
            src={TILE_ART[i]}
            alt=""
            aria-hidden="true"
            loading="lazy"
            decoding="async"
            fetchpriority="low"
          />
          <span class="tile-num">{i + 1}</span>
        </button>
      {/each}
    </div>
    <div class="flex gap-2">
      <button
        class="rounded-md bg-accent-candle px-4 py-2 text-sm font-medium text-bg"
        onclick={start}
        disabled={phase === 'showing'}
      >
        {phase === 'idle' ? 'Start' : 'Restart'}
      </button>
      <a
        class="inline-flex min-h-6 items-center self-center text-sm text-text-muted underline"
        href={resolve('/break')}
      >
        Back to the break room
      </a>
    </div>
  </div>
</AppShell>

<style>
  .tile-art {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    opacity: 0;
    transition: opacity 160ms ease-out;
    pointer-events: none;
  }
  .tile-reveal {
    border-color: var(--color-accent-candle, #f0a030);
  }
  .tile-reveal .tile-art {
    opacity: 1;
  }
  .tile-num {
    position: relative;
  }
  .tile-reveal .tile-num {
    opacity: 0.4;
  }
  .tile-correct {
    background: var(--color-pass, #4a9e6b);
  }
  .tile-wrong {
    background: var(--color-danger, #c04444);
  }
  /* The verdict colour washes through the revealed art rather than hiding it. */
  .tile-correct .tile-art,
  .tile-wrong .tile-art {
    opacity: 0.45;
  }
  @media (prefers-reduced-motion: reduce) {
    button,
    .tile-art {
      transition: none;
    }
  }
</style>
