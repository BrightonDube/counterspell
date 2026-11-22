// Tests for the dashboard request (dashboard-fsrs T09, DASH-1 and DASH-7). `fetch` is injected, so no test
// touches the network. Each test names the break it catches.
import { describe, expect, it } from 'vitest';
import { ApiRequestError } from '../api/client';
import { loadDashboard } from './client';

function jsonResponse(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' }
  });
}

const emptyDashboard = {
  solved: { total: 0, by_difficulty: { easy: 0, medium: 0, hard: 0 } },
  streak: { current_days: 0, active_today: false, last_activity_date: null },
  weak_concepts: [],
  recent_submissions: [],
  suggested_challenges: [],
  activity_days: []
};

describe('loadDashboard', () => {
  // Breaks if the page asks for another learner's dashboard or sends a user id from the browser.
  it('requests GET /api/dashboard with no user selector and the same-origin cookie policy', async () => {
    const calls: { url: string; init: RequestInit | undefined }[] = [];
    const fetchImpl = async (url: string, init?: RequestInit) => {
      calls.push({ url, init });
      return jsonResponse(200, emptyDashboard);
    };

    await loadDashboard({ fetchImpl });

    expect(calls).toHaveLength(1);
    expect(calls[0]?.url).toBe('/api/dashboard');
    expect(calls[0]?.init?.method ?? 'GET').toBe('GET');
    expect(calls[0]?.init?.credentials).toBe('same-origin');
  });

  // Breaks if a signed-out visitor sees a generic failure instead of the sign-in state.
  it('reports the server unauthenticated code for a 401', async () => {
    const fetchImpl = async () =>
      jsonResponse(401, {
        code: 'unauthenticated',
        message: 'Sign in to view your dashboard.'
      });

    const error = await loadDashboard({ fetchImpl }).catch(
      (caught: unknown) => caught
    );

    expect(error).toBeInstanceOf(ApiRequestError);
    expect((error as ApiRequestError).status).toBe(401);
    expect((error as ApiRequestError).code).toBe('unauthenticated');
  });

  // Breaks if a 503 is shown with a raw server body instead of the retryable message.
  it('reports dashboard_unavailable with a safe message for a 503', async () => {
    const fetchImpl = async () =>
      jsonResponse(503, {
        code: 'dashboard_unavailable',
        message: 'SELECT failed on relation submissions'
      });

    const error = (await loadDashboard({ fetchImpl }).catch(
      (caught: unknown) => caught
    )) as ApiRequestError;

    expect(error.status).toBe(503);
    expect(error.code).toBe('dashboard_unavailable');
    expect(error.message).not.toContain('SELECT');
  });

  // Breaks if an empty learner's response is turned into an error page.
  it('returns the parsed view for an empty learner', async () => {
    const fetchImpl = async () => jsonResponse(200, emptyDashboard);
    const view = await loadDashboard({ fetchImpl });
    expect(view.solved.total).toBe(0);
    expect(view.streak.activeToday).toBe(false);
  });
});
