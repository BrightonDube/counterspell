// Tests for the same-origin API adapters (ide-workspace T03). `fetch` is injected, so no test touches the network.
// Each test names the break it catches.
import { describe, expect, it } from 'vitest';
import { ApiRequestError, getJson } from './client';
import { loadLanguages } from './languages';
import { loadProblem } from './problems';

function jsonResponse(status: number, body: unknown): Response {
  return new Response(typeof body === 'string' ? body : JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' }
  });
}

const validProblem = {
  slug: 'two-sum',
  title: 'Two Sum',
  track: { slug: 'arrays', name: 'Crypt of Arrays' },
  concepts: [],
  difficulty: 'intro',
  statement_markdown: 'Find two indexes.',
  constraints: [],
  signature: {
    function_name: 'two_sum',
    parameters: [],
    return_type: 'list<int>'
  },
  samples: [],
  starters: { python: 'def two_sum(nums):\n    pass\n' },
  enabled_languages: ['python']
};

async function errorFrom(promise: Promise<unknown>): Promise<ApiRequestError> {
  try {
    await promise;
  } catch (error) {
    if (error instanceof ApiRequestError) return error;
    throw error;
  }
  throw new Error('expected the request to fail');
}

describe('getJson', () => {
  // Breaks if the request drops the same-origin cookie policy, so the session would not reach the server.
  it('sends a same-origin request with the session cookie policy', async () => {
    const calls: { url: string; init: RequestInit | undefined }[] = [];
    const fetchImpl = async (url: string, init?: RequestInit) => {
      calls.push({ url, init });
      return jsonResponse(200, validProblem);
    };
    await loadProblem('two-sum', { fetchImpl });
    expect(calls[0]?.url).toBe('/api/problems/two-sum');
    expect(calls[0]?.init?.credentials).toBe('same-origin');
  });

  // Breaks if an unauthenticated 401 is shown as a generic failure instead of a sign-in prompt.
  it('maps a 401 to unauthorized with the server safe message', async () => {
    const fetchImpl = async () =>
      jsonResponse(401, {
        code: 'unauthorized',
        message: 'Sign in to open this problem.'
      });
    const error = await errorFrom(loadProblem('two-sum', { fetchImpl }));
    expect(error.status).toBe(401);
    expect(error.code).toBe('unauthorized');
    expect(error.message).toBe('Sign in to open this problem.');
  });

  // Breaks if a 404 with a non-JSON body leaks the body (for example SQL or a stack trace) into the message.
  it('shows a fixed message for a 404 whose body is not an error envelope', async () => {
    const fetchImpl = async () =>
      new Response('SELECT * FROM problems: relation missing at line 4', {
        status: 404
      });
    const error = await errorFrom(loadProblem('two-sum', { fetchImpl }));
    expect(error.code).toBe('not_found');
    expect(error.message).not.toContain('SELECT');
    expect(error.message).not.toContain('relation');
  });

  // Breaks if a 500 with an internal body is passed through, since internal errors must stay generic.
  it('hides internal error details behind a generic unavailable message', async () => {
    const fetchImpl = async () =>
      jsonResponse(500, {
        code: 'internal',
        message: 'panic at worker 7: secret key abc'
      });
    const error = await errorFrom(loadProblem('two-sum', { fetchImpl }));
    expect(error.status).toBe(500);
    expect(error.code).toBe('internal');
    expect(error.message).not.toContain('secret');
  });

  // Breaks if a success body that is not JSON is treated as a valid problem.
  it('refuses a success body that is not JSON', async () => {
    const fetchImpl = async () =>
      new Response('<html>oops</html>', { status: 200 });
    const error = await errorFrom(loadProblem('two-sum', { fetchImpl }));
    expect(error.code).toBe('malformed_response');
  });

  // Breaks if JSON that fails the contract is rendered with guessed fields.
  it('refuses a JSON success body that fails the contract', async () => {
    const fetchImpl = async () => jsonResponse(200, { slug: 'two-sum' });
    const error = await errorFrom(loadProblem('two-sum', { fetchImpl }));
    expect(error.code).toBe('malformed_response');
  });

  // Breaks if a stalled request keeps the workspace in loading forever instead of failing with a retry.
  it('fails with a timeout when the server does not answer in time', async () => {
    const fetchImpl = (_url: string, init?: RequestInit) =>
      new Promise<Response>((_resolve, reject) => {
        init?.signal?.addEventListener('abort', () =>
          reject(new DOMException('aborted', 'AbortError'))
        );
      });
    const error = await errorFrom(
      getJson('/api/languages', (raw) => raw, { fetchImpl, timeoutMs: 5 })
    );
    expect(error.code).toBe('timeout');
  });

  // Breaks if an aborted request (the learner moved to another problem) still reports an error into the new page.
  it('reports an aborted request with the aborted code', async () => {
    const controller = new AbortController();
    const fetchImpl = (_url: string, init?: RequestInit) =>
      new Promise<Response>((_resolve, reject) => {
        init?.signal?.addEventListener('abort', () =>
          reject(new DOMException('aborted', 'AbortError'))
        );
      });
    const pending = errorFrom(
      loadProblem('two-sum', { fetchImpl, signal: controller.signal })
    );
    controller.abort();
    expect((await pending).code).toBe('aborted');
  });
});

describe('loadProblem', () => {
  // Breaks if a slug with path characters reaches the URL and changes which resource is requested.
  it('refuses a slug that is not a plain catalogue slug, without any request', async () => {
    let called = false;
    const fetchImpl = async () => {
      called = true;
      return jsonResponse(200, validProblem);
    };
    const error = await errorFrom(loadProblem('../admin', { fetchImpl }));
    expect(error.code).toBe('not_found');
    expect(called).toBe(false);
  });
});

describe('loadLanguages', () => {
  // Breaks if a language the server disabled reaches the picker because the adapter ignored the flag.
  it('keeps the server enabled flag for every language', async () => {
    const fetchImpl = async () =>
      jsonResponse(200, [
        {
          id: 'python',
          display_name: 'Python',
          enabled: true,
          file_name: 'solution.py',
          max_source_bytes: 65536,
          supports_run: true,
          supports_submit: true
        },
        {
          id: 'cpp',
          display_name: 'C++',
          enabled: false,
          file_name: 'solution.cpp',
          max_source_bytes: 65536,
          supports_run: false,
          supports_submit: false
        }
      ]);
    const languages = await loadLanguages({ fetchImpl });
    expect(
      languages.map((language) => [language.id, language.enabled])
    ).toEqual([
      ['python', true],
      ['cpp', false]
    ]);
  });
});
