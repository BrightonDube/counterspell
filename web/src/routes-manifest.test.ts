// test-change: PROD-FOUND-404
// Keeps the deployed SPA route manifest (`static/routes.json`) truthful: the
// server answers 200.html for exactly these paths and an honest 404 for
// everything else, so this file must mirror `src/routes`.
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

// vitest runs with `web/` as the working directory.
const routesDir = 'src/routes';
const manifestPath = 'static/routes.json';

function routesFromFileTree(): string[] {
  const routes: string[] = [];
  const walk = (dir: string, segments: string[]) => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      if (entry.isDirectory()) {
        let segment: string | null = entry.name;
        if (segment.startsWith('(') && segment.endsWith(')')) segment = null;
        else if (segment.startsWith('[...')) segment = '**';
        else if (segment.startsWith('[')) segment = '*';
        walk(
          join(dir, entry.name),
          segment === null ? segments : [...segments, segment]
        );
      } else if (entry.name === '+page.svelte') {
        routes.push('/' + segments.join('/'));
      }
    }
  };
  walk(routesDir, []);
  return routes.sort();
}

describe('static/routes.json', () => {
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8')) as {
    routes: string[];
  };

  it('lists exactly the routes present in src/routes', () => {
    expect([...manifest.routes].sort()).toEqual(routesFromFileTree());
  });

  it('contains only clean absolute route patterns', () => {
    for (const route of manifest.routes) {
      expect(route, route).toMatch(/^\//);
      expect(route, route).not.toContain('//');
      for (const segment of route.split('/').slice(1).filter(Boolean)) {
        expect(segment, route).not.toBe('');
        if (segment.includes('*')) {
          expect(['*', '**'], route).toContain(segment);
        }
      }
    }
  });
});
