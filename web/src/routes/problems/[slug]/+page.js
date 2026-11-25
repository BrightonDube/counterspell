// The workspace reads its problem from the API in the browser, so the slug page is not prerendered.
// First landings reuse the fetch the root layout already warmed (WP03); later
// navigations between problems fetch here, still at module evaluation rather
// than after the component mount chain.
import { loadLanguages } from '../../../lib/api/languages';
import { loadProblem } from '../../../lib/api/problems';

export const prerender = false;

export async function load({ params, parent }) {
  const { workspaceWarm } = await parent();
  const pending =
    workspaceWarm?.slug === params.slug
      ? workspaceWarm.promise
      : Promise.all([loadProblem(params.slug), loadLanguages()]);
  // Same unhandled-rejection guard as the layout warm path.
  pending.catch(() => {});
  return { pending };
}
