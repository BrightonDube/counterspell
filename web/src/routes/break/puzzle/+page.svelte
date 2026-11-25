<script>
  import AppShell from '../../../lib/shell/AppShell.svelte';
  import { resolve } from '$app/paths';
  import { onMount } from 'svelte';
  import piecesUrl from 'cm-chessboard/assets/pieces/standard.svg?url';
  import 'cm-chessboard/assets/chessboard.css';
  import { fetchDaily, saveScore } from '#lib/break/api.js';
  import { newPuzzle, tryMove } from '#lib/break/puzzle.js';
  import Seo from '../../../lib/seo/Seo.svelte';

  /** @type {HTMLDivElement | undefined} */
  let boardEl = $state();
  /** @type {import('cm-chessboard').Chessboard | undefined} */
  let board;
  /** @type {import('#lib/break/puzzle.js').PuzzleState | undefined} */
  let puzzleState = $state();
  /** @type {{ date: string, puzzle: { rating: number, themes: string[], moves: string, fen: string } } | undefined} */
  let puzzleMeta = $state();
  let feedback = $state('Loading puzzle…');
  let statusLine = $state('');
  let moveInput = $state('');
  let saveResult = $state('');
  let signedOut = $state(false);

  function refreshStatus() {
    if (!puzzleState) return;
    statusLine = `${puzzleState.game.turn() === 'w' ? 'White' : 'Black'} to move. FEN ${puzzleState.game.fen()}`;
  }

  /** @param {string} input */
  function apply(input) {
    if (!puzzleState) return;
    const before = puzzleState.game.fen();
    const { feedback: fb } = tryMove(puzzleState, input);
    feedback = fb;
    if (puzzleState.game.fen() !== before)
      board?.setPosition(puzzleState.game.fen(), true);
    refreshStatus();
    if (puzzleState.status === 'solved' && puzzleMeta) void recordScore();
  }

  async function recordScore() {
    if (!puzzleState || !puzzleMeta) return;
    const res = await saveScore({
      game: 'chess_puzzle',
      date: puzzleMeta.date,
      solved: true,
      mistakes: puzzleState.mistakes
    });
    if (res.ok) {
      saveResult =
        res.status === 'already_recorded'
          ? 'Already recorded today.'
          : `Score saved: ${res.score}.`;
    } else if (res.status === 401) {
      signedOut = true;
      saveResult = 'Sign in to save your score.';
    } else {
      saveResult = 'Score was not saved. You can retry.';
    }
  }

  function submitText() {
    const input = moveInput.trim();
    if (!input) return;
    moveInput = '';
    apply(input);
  }

  onMount(async () => {
    if (!boardEl) return;
    const daily = await fetchDaily();
    if (!daily.ok) {
      feedback =
        daily.code === 'not_imported'
          ? "Today's puzzle is not available yet."
          : 'Could not load the puzzle.';
      return;
    }
    puzzleMeta = /** @type {any} */ (daily);
    const solution = daily.puzzle.moves.split(' ');
    puzzleState = newPuzzle(daily.puzzle.fen, solution);
    // cm-chessboard ships `main: null`, so Node cannot resolve it — importing it
    // lazily keeps the route server-renderable (Seo head + static markup).
    const { Chessboard, INPUT_EVENT_TYPE } = await import('cm-chessboard');
    if (!boardEl?.isConnected) return;
    board = new Chessboard(boardEl, {
      position: puzzleState.game.fen(),
      style: { pieces: { file: piecesUrl } }
    });
    // The container already carries the accessible name; the injected svg is
    // decorative chrome and must not surface as a second, unnamed img.
    boardEl.querySelector('svg')?.setAttribute('aria-hidden', 'true');
    board.enableMoveInput((event) => {
      if (event.type !== INPUT_EVENT_TYPE.validateMoveInput) return true;
      if (!puzzleState) return false;
      const uci = (event.squareFrom ?? '') + (event.squareTo ?? '');
      const ok = puzzleState.game
        .moves({ verbose: true })
        .some(
          (/** @type {{ from: string, to: string }} */ m) =>
            m.from + m.to === uci
        );
      if (ok) apply(uci);
      return ok;
    });
    refreshStatus();
    feedback = 'Your move.';
  });
</script>

<Seo
  title="Daily puzzle · Counterspell"
  description="One chess puzzle a day, picked for you. Solve it in the break room."
  path="/break/puzzle"
/>

<AppShell>
  <div class="mx-auto max-w-2xl space-y-4">
    <h1 class="text-2xl font-bold">Daily chess puzzle</h1>
    {#if puzzleMeta}
      <p class="text-sm text-text-muted">
        {puzzleMeta.date} · rating {puzzleMeta.puzzle.rating} · {puzzleMeta.puzzle.themes.join(
          ', '
        )}
        {#if puzzleMeta.puzzle.themes.some( (/** @type {string} */ t) => t.startsWith('mate') )}
          · Hint: look for a mate.
        {/if}
      </p>
    {/if}
    <div
      bind:this={boardEl}
      class="break-board max-w-md"
      data-testid="board"
      role="img"
      aria-label="Chessboard"
    ></div>
    <output class="block text-sm text-text-muted" aria-live="polite">
      {statusLine}
    </output>
    <p role="status" class="text-sm">{feedback}</p>
    {#if saveResult}<p role="status" class="text-sm text-text-muted">
        {saveResult}
      </p>{/if}
    {#if signedOut}<a
        class="text-accent underline"
        href={resolve('/login?return_to=/break/puzzle')}
      >
        Sign in
      </a>{/if}
    <form
      class="flex gap-2"
      onsubmit={(e) => {
        e.preventDefault();
        submitText();
      }}
    >
      <label class="sr-only" for="move-input">
        Move in UCI (e2e4) or SAN (Nf3)
      </label>
      <input
        id="move-input"
        class="w-40 rounded-md border border-border bg-surface px-3 py-2 text-sm"
        bind:value={moveInput}
        placeholder="e2e4 or Nf3"
        autocomplete="off"
      />
      <button
        type="submit"
        class="rounded-md bg-accent-candle px-4 py-2 text-sm font-medium text-bg hover:bg-accent-candle-glow"
      >
        Play
      </button>
    </form>
    <a
      class="inline-flex min-h-6 items-center text-sm text-text-muted underline"
      href={resolve('/break')}
    >
      Back to the break room
    </a>
  </div>
</AppShell>
