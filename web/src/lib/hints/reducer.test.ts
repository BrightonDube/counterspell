// Tests for the hint ladder reducer (hints spec H15). The reducer is pure, so no DOM is needed.
// Each test names the break it catches.
import { describe, expect, it } from 'vitest';
import {
  canRequest,
  hintReducer,
  initialHintState,
  type HintAction
} from './reducer';
import type { HintReply } from './client';

function ladderReply(
  rung: number,
  body: string,
  nextRung: number | null
): HintReply {
  return {
    rung,
    source: 'ladder',
    body,
    label: null,
    provider: null,
    model: null,
    sentToProvider: false,
    quota: null,
    nextRung
  };
}

function run(actions: HintAction[]) {
  return actions.reduce(hintReducer, initialHintState());
}

describe('canRequest', () => {
  // Breaks if rung 3 can be asked for before rung 2 is revealed, which skips the ladder.
  it('refuses rung 3 until rung 2 is revealed', () => {
    const start = run([
      { type: 'start', rung: 1, ai: false },
      { type: 'reply', reply: ladderReply(1, 'Look at the loop.', 2) }
    ]);

    expect(canRequest(start, 3, false)).toBe(false);
    expect(canRequest(start, 2, false)).toBe(true);
  });

  // Breaks if a rung that already has text is requested again, which would spend a request for nothing.
  it('refuses a rung that is already revealed', () => {
    const state = run([
      { type: 'start', rung: 1, ai: false },
      { type: 'reply', reply: ladderReply(1, 'Look at the loop.', 2) }
    ]);

    expect(canRequest(state, 1, false)).toBe(false);
  });

  // Breaks if an AI hint is offered for rung 1, which the server refuses (AI starts at rung 2).
  it('refuses an AI hint for rung 1', () => {
    expect(canRequest(initialHintState(), 1, true)).toBe(false);
  });

  // Breaks if a second request starts while one is in flight, which would race two replies.
  it('refuses a new request while one is pending', () => {
    const pending = hintReducer(initialHintState(), {
      type: 'start',
      rung: 1,
      ai: false
    });

    expect(canRequest(pending, 1, false)).toBe(false);
  });
});

describe('hintReducer', () => {
  // Breaks if a reply does not record its text under its rung, so the panel shows nothing.
  it('stores a ladder reply under its rung and moves on', () => {
    const state = run([
      { type: 'start', rung: 1, ai: false },
      { type: 'reply', reply: ladderReply(1, 'Look at the loop.', 2) }
    ]);

    expect(state.ladder[1]).toBe('Look at the loop.');
    expect(state.nextRung).toBe(2);
    expect(state.pending).toBeNull();
  });

  // Breaks if the AI label and provider are dropped, which would present AI text as authored.
  it('stores an AI reply with its label and the quota it reports', () => {
    const state = run([
      { type: 'start', rung: 1, ai: false },
      { type: 'reply', reply: ladderReply(1, 'Look at the loop.', 2) },
      { type: 'start', rung: 2, ai: true },
      {
        type: 'reply',
        reply: {
          rung: 2,
          source: 'ai',
          body: 'Check the second loop.',
          label: 'AI hint',
          provider: 'openai',
          model: 'gpt-6-luna',
          sentToProvider: true,
          quota: { used: 1, limit: 10, resetsAt: '2026-11-17T00:00:00Z' },
          nextRung: 3
        }
      }
    ]);

    expect(state.ai[2]).toEqual({
      body: 'Check the second loop.',
      label: 'AI hint',
      provider: 'openai',
      model: 'gpt-6-luna'
    });
    expect(state.quota).toEqual({
      used: 1,
      limit: 10,
      resetsAt: '2026-11-17T00:00:00Z'
    });
  });

  // Breaks if a failed request clears what the learner already saw, or leaves the panel stuck pending.
  it('keeps revealed rungs and clears pending when a request fails', () => {
    const state = run([
      { type: 'start', rung: 1, ai: false },
      { type: 'reply', reply: ladderReply(1, 'Look at the loop.', 2) },
      { type: 'start', rung: 2, ai: false },
      { type: 'failed', message: 'The hint could not be loaded.' }
    ]);

    expect(state.ladder[1]).toBe('Look at the loop.');
    expect(state.ladder[2]).toBeUndefined();
    expect(state.pending).toBeNull();
    expect(state.error).toBe('The hint could not be loaded.');
  });

  // Breaks if a request that the gate refuses still changes the state, so the panel shows a rung it never got.
  it('ignores a start that canRequest refuses', () => {
    const before = initialHintState();

    const after = hintReducer(before, { type: 'start', rung: 3, ai: false });

    expect(after).toEqual(before);
  });
});
