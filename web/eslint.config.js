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
    ignores: ['.svelte-kit/**', 'build/**', 'node_modules/**']
  }
);
