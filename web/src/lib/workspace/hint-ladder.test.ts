// Tests for the authored hint ladder state (ide-workspace T17). Each test names the break it catches.
import { describe, expect, it } from 'vitest';
import {
  failRung,
  initialLadder,
  markCapReached,
  receiveRung,
  requestNext
} from './hint-ladder';

describe('requestNext', () => {
  // Breaks if the first request asks for a later rung, which would reveal a hint the learner did not ask for in order.
  it('asks for rung 1 first', () => {
    const result = requestNext(initialLadder(3));
    expect(result.rung).toBe(1);
  });

  // Breaks if a second click sends a duplicate request for the same rung while the first is still pending.
  it('does not request again while a rung is pending', () => {
    const first = requestNext(initialLadder(3));
    expect(requestNext(first.state).rung).toBeNull();
  });

  // Breaks if the ladder asks for rungs beyond the authored total.
  it('stops asking once every authored rung is revealed', () => {
    let state = initialLadder(1);
    state = requestNext(state).state;
    state = receiveRung(state, 1, 'Nudge.');
    expect(requestNext(state).rung).toBeNull();
  });
});

describe('receiveRung', () => {
  // Breaks if rungs appear out of order, so a late answer to rung 2 shows before rung 1.
  it('ignores a rung that is not the one being requested', () => {
    const pending = requestNext(initialLadder(3)).state;
    const after = receiveRung(pending, 2, 'Targeted fix.');
    expect(after.revealed).toEqual([]);
  });

  // Breaks if a valid rung is dropped or its text is changed on the way to the learner.
  it('reveals the requested rung with its exact text', () => {
    const pending = requestNext(initialLadder(3)).state;
    const after = receiveRung(pending, 1, 'Can an index pair with itself?');
    expect(after.revealed).toEqual([
      { rung: 1, text: 'Can an index pair with itself?' }
    ]);
  });
});

describe('failure and cap states', () => {
  // Breaks if a failed request erases the hints the learner already read.
  it('keeps revealed hints when a later request fails', () => {
    let state = requestNext(initialLadder(3)).state;
    state = receiveRung(state, 1, 'Nudge.');
    state = requestNext(state).state;
    state = failRung(state, 2, 'The hint could not load. Try again.');
    expect(state.revealed).toEqual([{ rung: 1, text: 'Nudge.' }]);
    expect(state.error).toBe('The hint could not load. Try again.');
  });

  // Breaks if a failed request shows made-up hint text in place of the missing rung.
  it('does not invent text for a failed rung', () => {
    const state = failRung(
      requestNext(initialLadder(3)).state,
      1,
      'Try again.'
    );
    expect(state.revealed).toEqual([]);
  });

  // Breaks if the cap is shown as a silent stop, so the learner cannot tell why no more AI hints arrive.
  it('records that the daily cap was reached and stops requesting', () => {
    const state = markCapReached(requestNext(initialLadder(3)).state);
    expect(state.capReached).toBe(true);
    expect(requestNext(state).rung).toBeNull();
  });
});
