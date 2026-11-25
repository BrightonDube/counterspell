<script>
  import AppShell from '../../../lib/shell/AppShell.svelte';
  import { resolve } from '$app/paths';
  import { onMount, onDestroy } from 'svelte';
  import { Chess } from 'chess.js';
  import 'cm-chessboard/assets/chessboard.css';
  import {
    clearLastMove,
    mountThemedBoard,
    showLastMove
  } from '#lib/break/chess/theme.js';
  import SkullSentinel from '#lib/break/chess/SkullSentinel.svelte';
  import { saveScore } from '#lib/break/api.js';
  import { session } from '#lib/auth/session.svelte.js';
  import BotWorker from '#lib/break/bot.worker.js?worker';
  import Seo from '../../../lib/seo/Seo.svelte';

  /** @type {HTMLDivElement | undefined} */
  let boardEl = $state();
  let game = new Chess();
  /** @type {import('cm-chessboard').Chessboard | undefined} */
  let board;
  /** @type {Worker | undefined} */
  let worker = $state();
  /** @type {'w' | 'b'} */
  let userColor = $state('w');
  let status = $state('Pick a colour to start.');
  let statusLine = $state('');
  let playing = $state(false);
  let fallbackNote = $state('');
  let seed = String(Date.now());

  function refreshStatus() {
    statusLine = `${game.turn() === 'w' ? 'White' : 'Black'} to move. FEN ${game.fen()}`;
    if (game.isCheckmate()) {
      playing = false;
      const won = game.turn() !== userColor;
      status = won ? 'You win. Well played.' : 'The bot wins this one.';
      void recordEnd({
        game: 'chess_bot',
        result: won ? 'win' : 'loss',
        moves: game.moveNumber()
      });
    } else if (game.isDraw() || game.isStalemate()) {
      playing = false;
      status = 'A draw.';
      void recordEnd({
        game: 'chess_bot',
        result: 'draw',
        moves: game.moveNumber()
      });
    }
  }

  // Await the session answer before deciding — a probe still in flight must
  // not drop a signed-in player's result (AUD-19/AUD-20).
  /** @param {Record<string, unknown>} payload */
  function recordEnd(payload) {
    void session
      .load()
      .then(() => {
        if (session.status === 'signed_in') void saveScore(payload);
      })
      .catch(() => {});
  }

  function botMove() {
    if (!playing) return;
    if (!worker) {
      const moves = game.moves({ verbose: true });
      const pick = moves[Math.floor(Math.random() * moves.length)];
      if (pick) {
        game.move(pick);
        showLastMove(board, pick.from, pick.to);
      }
      fallbackNote = 'Bot is resting, a simple move was played.';
      board?.setPosition(game.fen(), true);
      refreshStatus();
      return;
    }
    worker.onmessage = (event) => {
      const data = event.data || {};
      if (data.ok && data.uci) {
        game.move({
          from: data.uci.slice(0, 2),
          to: data.uci.slice(2, 4),
          promotion: data.uci[4]
        });
        showLastMove(board, data.uci.slice(0, 2), data.uci.slice(2, 4));
      } else {
        const moves = game.moves({ verbose: true });
        const pick = moves[Math.floor(Math.random() * moves.length)];
        if (pick) {
          game.move(pick);
          showLastMove(board, pick.from, pick.to);
        }
        fallbackNote = 'Bot is resting, a simple move was played.';
      }
      board?.setPosition(game.fen(), true);
      refreshStatus();
    };
    worker.postMessage({ fen: game.fen(), seed });
  }

  /** @param {'w' | 'b'} color */
  function start(color) {
    userColor = color;
    game = new Chess();
    clearLastMove(board);
    board?.setPosition(game.fen());
    playing = true;
    status = color === 'w' ? 'Your move.' : 'The bot opens.';
    refreshStatus();
    if (game.turn() !== userColor) botMove();
  }

  onDestroy(() => worker?.terminate());

  onMount(async () => {
    // Session copy is honest about who is playing: anonymous players see the
    // sign-in nudge; signed-in players never get told their result is unsaved.
    void session.load().catch(() => {});
    if (!boardEl) return;
    const container = boardEl;
    // Shared Steel Night mount: theme class, piece sprite, markers extension
    // and the aria-hidden svg fix all live in #lib/break/chess/theme.js (CH02–CH05).
    const { board: themed, INPUT_EVENT_TYPE } = await mountThemedBoard(
      container,
      game.fen()
    );
    if (!themed) return;
    try {
      worker = new BotWorker();
    } catch {
      worker = undefined;
    }
    board = themed;
    board.enableMoveInput((event) => {
      if (event.type !== INPUT_EVENT_TYPE.validateMoveInput) return true;
      if (!playing || game.turn() !== userColor) return false;
      const move = game.move({
        from: event.squareFrom ?? '',
        to: event.squareTo ?? '',
        promotion: 'q'
      });
      if (!move) return false;
      showLastMove(board, move.from, move.to);
      board?.setPosition(game.fen(), true);
      refreshStatus();
      if (playing && game.turn() !== userColor) setTimeout(botMove, 50);
      return true;
    });
    refreshStatus();
  });
</script>

<Seo
  title="Chess bot · Counterspell"
  description="A casual game of chess against a small local engine in the break room."
  path="/break/bot"
/>

<AppShell>
  <div class="mx-auto max-w-3xl space-y-4">
    <h1 class="text-2xl font-bold">Chess bot</h1>
    <div class="flex gap-2">
      <button
        class="rounded-md bg-accent-candle px-4 py-2 text-sm font-medium text-bg"
        onclick={() => start('w')}
      >
        Play White
      </button>
      <button
        class="rounded-md border border-border px-4 py-2 text-sm"
        onclick={() => start('b')}
      >
        Play Black
      </button>
    </div>
    {#if playing}
      <SkullSentinel />
    {/if}
    <div
      bind:this={boardEl}
      class="break-board"
      role="img"
      aria-label="Chessboard"
    ></div>
    <output class="block text-sm text-text-muted" aria-live="polite">
      {statusLine}
    </output>
    <p role="status" class="text-sm">{status}</p>
    {#if fallbackNote}<p role="status" class="text-sm text-text-muted">
        {fallbackNote}
      </p>{/if}
    {#if session.status === 'signed_out'}
      <p class="text-sm text-text-muted">
        <a
          class="text-accent underline"
          href={resolve('/login?return_to=/break/bot')}
        >
          Sign in
        </a>
        to save your results.
      </p>
    {/if}
    <a
      class="inline-flex min-h-6 items-center text-sm text-text-muted underline"
      href={resolve('/break')}
    >
      Back to the break room
    </a>
  </div>
</AppShell>
