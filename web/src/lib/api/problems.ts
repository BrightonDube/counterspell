// Loads one published problem from `GET /api/problems/{slug}`. Problem content is never bundled.
import { ApiRequestError, getJson, type RequestOptions } from './client';
import { parseProblemDetail, type ProblemDetail } from './types';

/** Catalogue slugs are lowercase letters, digits and hyphens. Anything else is refused before a request. */
const SLUG_PATTERN = /^[a-z0-9][a-z0-9-]{0,127}$/;

/** Fetches a problem by slug. A malformed slug fails locally and sends no request. */
export function loadProblem(
  slug: string,
  options: RequestOptions = {}
): Promise<ProblemDetail> {
  if (!SLUG_PATTERN.test(slug)) {
    return Promise.reject(
      new ApiRequestError(404, 'not_found', 'That problem does not exist.')
    );
  }
  return getJson(
    `/api/problems/${encodeURIComponent(slug)}`,
    parseProblemDetail,
    options
  );
}
