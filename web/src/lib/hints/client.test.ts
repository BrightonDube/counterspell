// Tests for the hints client (hints spec H15). `fetchImpl` is injected, so no test reaches the network.
// Each test names the break it catches.
import { describe, expect, it } from 'vitest';
import { ApiRequestError } from '../api/client';
import { requestHint } from './client';

const SUBMISSION = '11111111-1111-4111-8111-111111111111';

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' }
  });
}

function ladderReply(overrides: Record<string, unknown> = {}) {
  return {
    rung: 1,
    source: 'ladder',
    body: 'Look at the loop bounds.',
    label: null,
    provider: null,
    model: null,
    sent_to_provider: false,
    quota: null,
    next_rung: 2,
    ...overrides
  };
}

async function failureOf(promise: Promise<unknown>): Promise<ApiRequestError> {
  try {
    await promise;
  } catch (error) {
    if (error instanceof ApiRequestError) return error;
    throw error;
  }
  throw new Error('expected the request to fail');
}

describe('requestHint', () => {
  // Breaks if the request body is not the server's shape, so the server reads the wrong rung or mode.
  it('posts the submission, the rung and the ai flag as JSON to /api/hints', async () => {
    let sentUrl = '';
    let sentInit: RequestInit | undefined;
    const fetchImpl = async (url: string, init?: RequestInit) => {
      sentUrl = url;
      sentInit = init;
      return jsonResponse(ladderReply());
    };

    await requestHint(
      { submissionId: SUBMISSION, rung: 1, ai: false },
      { fetchImpl }
    );

    expect(sentUrl).toBe('/api/hints');
    expect(sentInit?.method).toBe('POST');
    expect(sentInit?.credentials).toBe('same-origin');
    expect(JSON.parse(String(sentInit?.body))).toEqual({
      submission_id: SUBMISSION,
      rung: 1,
      ai: false
    });
  });

  // Breaks if a reply for another rung is accepted, which would show the wrong hint under a rung.
  it('refuses a reply for a different rung', async () => {
    const fetchImpl = async () => jsonResponse(ladderReply({ rung: 3 }));

    const failure = await failureOf(
      requestHint(
        { submissionId: SUBMISSION, rung: 1, ai: false },
        { fetchImpl }
      )
    );

    expect(failure).toBeInstanceOf(ApiRequestError);
  });

  // Breaks if the client turns hint text into markup, so a provider reply could run script.
  it('keeps hint text verbatim, so it is shown as plain text', async () => {
    const text = '<img src=x onerror=alert(1)> use a set';
    const fetchImpl = async () => jsonResponse(ladderReply({ body: text }));

    const reply = await requestHint(
      { submissionId: SUBMISSION, rung: 1, ai: false },
      { fetchImpl }
    );

    expect(reply.body).toBe(text);
  });

  // Breaks if the AI label or the provider flag is lost, which would hide that the text is AI-written.
  it('keeps the AI label, provider and sent-to-provider flag on an AI reply', async () => {
    const fetchImpl = async () =>
      jsonResponse({
        rung: 2,
        source: 'ai',
        body: 'Check the second loop.',
        label: 'AI hint',
        provider: 'openai',
        model: 'gpt-6-luna',
        sent_to_provider: true,
        quota: { used: 1, limit: 10, resets_at: '2026-11-17T00:00:00Z' },
        next_rung: 3
      });

    const reply = await requestHint(
      { submissionId: SUBMISSION, rung: 2, ai: true },
      { fetchImpl }
    );

    expect(reply.source).toBe('ai');
    expect(reply.label).toBe('AI hint');
    expect(reply.provider).toBe('openai');
    expect(reply.sentToProvider).toBe(true);
    expect(reply.quota).toEqual({
      used: 1,
      limit: 10,
      resetsAt: '2026-11-17T00:00:00Z'
    });
    expect(reply.nextRung).toBe(3);
  });

  // Breaks if a rung outside 1 to 4 reaches the server, which would waste a request.
  it('refuses a rung outside 1 to 4 before any request is sent', async () => {
    let called = false;
    const fetchImpl = async () => {
      called = true;
      return jsonResponse(ladderReply());
    };

    await expect(
      requestHint(
        { submissionId: SUBMISSION, rung: 5, ai: false },
        { fetchImpl }
      )
    ).rejects.toThrow(TypeError);
    expect(called).toBe(false);
  });
});
