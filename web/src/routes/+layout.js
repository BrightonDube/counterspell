import { loadLanguages } from '../lib/api/languages';
import { loadProblem } from '../lib/api/problems';

export const prerender = true;

// The workspace's problem fetch is its critical path, but the route module only
// evaluates one serial hop after this layout. Starting the request here — when
// the first landing on a problem page happens — overlaps the fetch with the
// page module's download instead of waiting for it (WP03). Later slug changes
// do not re-run this load; the page falls back to fetching its own data.
export function load({ route, params }) {
  if (route.id === '/problems/[slug]' && params.slug) {
    const promise = Promise.all([loadProblem(params.slug), loadLanguages()]);
    // The page attaches its real handlers after its module loads; silence the
    // "unhandled rejection" window in between — awaiting `promise` still sees
    // the failure.
    promise.catch(() => {});
    return { workspaceWarm: { slug: params.slug, promise } };
  }
  return {};
}
