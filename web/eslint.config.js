import { loadConfig } from '@sveltejs/load-config';
import prettier from 'eslint-config-prettier';
import js from '@eslint/js';
import { defineConfig } from 'eslint/config';
import globals from 'globals';
import svelte from 'eslint-plugin-svelte';
import ts from 'typescript-eslint';

const svelteConfig = (await loadConfig('./', { traverse: false }))?.config;

export default defineConfig(
  js.configs.recommended,
  ...ts.configs.strict,
  ...svelte.configs.recommended,
  ...svelte.configs.prettier,
  prettier,
  {
    languageOptions: {
      parserOptions: {
        extraFileExtensions: ['.svelte']
      },
      globals: {
        ...globals.browser,
        ...globals.node
      }
    },
    rules: {
      eqeqeq: ['error', 'smart'],
      'no-console': ['error', { allow: ['warn', 'error'] }],
      'no-unused-vars': 'off',
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' }
      ]
    }
  },
  {
    files: ['**/*.svelte', '**/*.svelte.js', '**/*.svelte.ts'],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
        svelteConfig
      }
    }
  },
  {
    // Node command-line harness scripts. Their job is to print results, so console output is
    // allowed here. Application code under src/ keeps the no-console rule.
    files: ['e2e-*.mjs', 'tests/real-api/**/*.mjs'],
    rules: {
      'no-console': 'off'
    }
  },
  {
    ignores: [
      '.svelte-kit/**',
      'build/**',
      'node_modules/**',
      // Git-ignored runtime artifacts of the real-API harness (see .gitignore).
      'tests/real-api/.scratch/**'
    ]
  }
);
