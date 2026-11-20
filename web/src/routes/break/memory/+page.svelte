<script>
  import { onMount } from 'svelte';
  import { checkPress, level, newRound } from '#lib/break/memory.js';
  import { saveScore } from '#lib/break/api.js';

  const TILES = 9;
  /** @type {number[]} */
  let sequence = $state([]);
  /** @type {number[]} */
  let presses = $state([]);
  let lit = $state(-1);
  /** @type {'idle' | 'showing' | 'input' | 'over'} */
  let phase = $state('idle');
  let announce = $state('Press start to play Lantern recall.');
  let flashTile = $state(-1);
  /** @type {'' | 'correct' | 'wrong'} */
  let flashKind = $state('');
  let focusIdx = $state(0);
  /** @type {HTMLButtonElement[]} */
  let tileEls = [];
  let reduced = false;

  async function showSequence() {
    phase = 'showing';
    if (reduced) {
      announce = 'Sequence: ' + sequence.map((t) => `tile ${t + 1}`).join(', ') + '.';
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
    flashTile = i;
    if (verdict === 'wrong') {
      flashKind = 'wrong';
      phase = 'over';
      const lvl = level(sequence);
      announce = `Wrong, game over at level ${lvl}.`;
      void saveScore({ game: 'brain', level: lvl });
    } else {
      flashKind = 'correct';
      announce = verdict === 'round_done' ? 'Correct. Next round.' : 'Correct.';
      if (verdict === 'round_done') {
        sequence = newRound(sequence);
        setTimeout(showSequence, 800);
      }
    }
    setTimeout(() => (flashTile = -1), 400);
  }

  /** @param {KeyboardEvent} event @param {number} i */
  function onKey(event, i) {
    const move = /** @type {Record<string, number>} */ ({
      ArrowRight: 1,
      ArrowLeft: -1,
      ArrowDown: 3,
      ArrowUp: -3,
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

<svelte:head><title>Lantern recall - Break room</title></svelte:head>

<main class="mx-auto max-w-md space-y-4 p-6">
  <h1 class="text-2xl font-bold">Lantern recall</h1>
  <p class="text-sm text-text-muted">Watch the tiles light up, then repeat the order.</p>
  <div aria-live="polite" class="min-h-6 text-sm">{announce}</div>
  <div class="grid grid-cols-3 gap-2" role="group" aria-label="Memory tiles">
    {#each Array(TILES) as _, i (i)}
      <button
        bind:this={tileEls[i]}
        type="button"
        tabindex={focusIdx === i ? 0 : -1}
        onfocus={() => (focusIdx = i)}
        onclick={() => press(i)}
        onkeydown={(e) => onKey(e, i)}
        class="aspect-square rounded-lg border text-sm font-medium transition-colors
          {lit === i ? 'tile-lit' : ''}
          {flashTile === i && flashKind === 'correct' ? 'tile-correct' : ''}
          {flashTile === i && flashKind === 'wrong' ? 'tile-wrong' : ''}
          {phase === 'input' ? 'border-border bg-surface' : 'border-border bg-raised'}"
        aria-label={'Tile ' + (i + 1)}
        >{i + 1}</button
      >
    {/each}
  </div>
  <div class="flex gap-2">
    <button
      class="rounded-md bg-accent-candle px-4 py-2 text-sm font-medium text-bg"
      onclick={start}
      disabled={phase === 'showing'}>{phase === 'idle' ? 'Start' : 'Restart'}</button
    >
    <a class="self-center text-sm text-text-muted underline" href="/break">Back to the break room</a>
  </div>
</main>

<style>
  .tile-lit {
    background: var(--color-accent-candle, #f0a030);
  }
  .tile-correct {
    background: var(--color-pass, #4a9e6b);
  }
  .tile-wrong {
    background: var(--color-danger, #c04444);
  }
  @media (prefers-reduced-motion: reduce) {
    button {
      transition: none;
    }
  }
</style>
