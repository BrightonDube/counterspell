import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [
    tailwindcss(),
    sveltekit({ adapter: adapter({ fallback: '200.html' }) })
  ],
  // Vitest must resolve the client Svelte build, not the SSR one. `browser`
  // mainFields matter: cm-chessboard ships `main: null` and only exposes its
  // entry through the browser/module fields.
  resolve: process.env.VITEST
    ? {
        conditions: ['browser'],
        mainFields: ['browser', 'module', 'jsnext:main']
      }
    : undefined,
  test: {
    environment: 'happy-dom',
    include: ['src/**/*.{test,spec}.{js,ts}']
  }
});
