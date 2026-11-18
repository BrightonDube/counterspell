// Viewport breakpoints from the workspace design: desktop is a two-column shell at
// 1200px and up, tablet (768-1199px) keeps the editor visible while problem and results
// become drawers, and narrow (<768px) stacks every region in one column.
import { browser } from '$app/env';

export type LayoutMode = 'desktop' | 'tablet' | 'narrow';

const DESKTOP_QUERY = '(min-width: 1200px)';
const TABLET_QUERY = '(min-width: 768px)';

/** Tracks the live viewport bucket. Defaults to `desktop` when matchMedia is absent. */
export function createLayoutMode(): { readonly mode: LayoutMode } {
  let mode = $state<LayoutMode>('desktop');
  if (browser && typeof window.matchMedia === 'function') {
    const desktop = window.matchMedia(DESKTOP_QUERY);
    const tablet = window.matchMedia(TABLET_QUERY);
    const update = (): void => {
      mode = desktop.matches ? 'desktop' : tablet.matches ? 'tablet' : 'narrow';
    };
    update();
    desktop.addEventListener('change', update);
    tablet.addEventListener('change', update);
  }
  return {
    get mode() {
      return mode;
    }
  };
}
