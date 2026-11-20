// The once-per-run speech request. One server call per run id, however often the page remounts or the
// event stream reconnects. The memory lives for the page's session. Speech is words only: nothing here
// can set a verdict or a mood.
import {
  requestSpeech,
  type SpeechPreference,
  type SpeechResult
} from '../../api/familiar';
import type { FetchLike } from '../../api/client';

/** Asks for the speech bubble of finished runs, at most once per run. */
export interface BubbleRequester {
  /**
   * Returns the run's bubble, or `null` when speech is off (nothing is sent). The same promise comes
   * back for every later call with the same run id, including after a failure.
   */
  request(
    runId: string,
    speech: SpeechPreference
  ): Promise<SpeechResult | null>;
}

/** Builds a requester with its own memory. Tests pass a fake `fetch`. */
export function createBubbleRequester(fetchImpl?: FetchLike): BubbleRequester {
  const asked = new Map<string, Promise<SpeechResult>>();
  return {
    request(runId, speech) {
      if (speech === 'off') return Promise.resolve(null);
      const existing = asked.get(runId);
      if (existing !== undefined) return existing;
      const pending = requestSpeech(
        runId,
        fetchImpl === undefined ? {} : { fetchImpl }
      );
      asked.set(runId, pending);
      return pending;
    }
  };
}

/** The page-wide requester. It survives remounts because it lives in this module. */
export const bubbleRequester: BubbleRequester = createBubbleRequester();

/** The `FamiliarDeps.requestSpeech` the workspace uses. The shell checks the speech preference first. */
export function requestSpeechOnce(runId: string): Promise<SpeechResult> {
  return bubbleRequester
    .request(runId, 'auto')
    .then((result) => result as SpeechResult);
}
