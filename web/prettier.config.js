/** @type {import('prettier').Config} */
export default {
  singleQuote: true,
  trailingComma: 'none',
  plugins: ['prettier-plugin-svelte'],
  overrides: [{ files: '*.svelte', options: { parser: 'svelte' } }]
};
