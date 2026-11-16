// Loads the signed-in learner's dashboard from `GET /api/dashboard`. The request has no user selector: the server
// takes the owner from the session cookie.
import { getJson, type RequestOptions } from '../api/client';
import { parseDashboard, type DashboardView } from './types';

/** Fetches the dashboard. Failures are `ApiRequestError`s with the server's stable code. */
export function loadDashboard(
  options: RequestOptions = {}
): Promise<DashboardView> {
  return getJson('/api/dashboard', parseDashboard, options);
}
