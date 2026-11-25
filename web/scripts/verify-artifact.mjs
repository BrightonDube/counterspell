// Build-artifact policy check (WP09): the production bundle must not carry
// source maps — no *.map files, and no sourceMappingURL references. Runs as the
// last step of `npm run build`, so CI and local builds enforce it alike.
import { readdir, readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';

const build = resolve('build');
const offenders = [];

async function scan(dir) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      await scan(path);
    } else if (entry.name.endsWith('.map')) {
      offenders.push(`${path}: source-map file shipped`);
    } else if (/\.(js|css|html)$/.test(entry.name)) {
      const text = await readFile(path, 'utf8');
      if (text.includes('sourceMappingURL=')) {
        offenders.push(`${path}: references a source map`);
      }
    }
  }
}

await scan(build);

if (offenders.length > 0) {
  console.error('Production source-map policy violated (WP09):');
  for (const line of offenders) console.error(`  ${line}`);
  process.exit(1);
}
console.warn('artifact check: no source maps in build output (WP09)');
