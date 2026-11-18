// Stamps approval.toml for a test pack pair, mirroring the judge's
// build_manifest: sha256 every file, join "tree/path\0hex\n" in path order,
// hash the canonical bytes. approval.toml itself is never hashed.
//
// The private tree must live outside the public repository (the importer's
// preflight refuses anything nested in it), so this script materializes the
// committed template at private-src/ into a scratch root and stamps it there.
// Usage: node stamp-approval.mjs <slug> <scratch-private-root>
import { createHash } from 'node:crypto';
import { execSync } from 'node:child_process';
import {
  cpSync,
  readdirSync,
  readFileSync,
  writeFileSync,
  statSync
} from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = fileURLToPath(new URL('.', import.meta.url));
const slug = process.argv[2] ?? 'add-two';
const scratchRoot = process.argv[3];
if (!scratchRoot) {
  console.error('usage: node stamp-approval.mjs <slug> <scratch-private-root>');
  process.exit(2);
}

// Materialize the template into the scratch private root (outside this repo).
cpSync(join(here, 'private-src'), scratchRoot, { recursive: true });

const trees = [
  ['public', join(here, 'packs', slug)],
  ['private', join(scratchRoot, 'packs', slug)]
];

const sha = (bytes) => createHash('sha256').update(bytes).digest('hex');

function walk(dir, base = dir) {
  const out = [];
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) out.push(...walk(p, base));
    else out.push(relative(base, p).split('\\').join('/'));
  }
  return out;
}

const entries = [];
for (const [tree, root] of trees) {
  for (const rel of walk(root)) {
    if (rel === 'approval.toml') continue;
    entries.push({
      path: `${tree}/${rel}`,
      hex: sha(readFileSync(join(root, rel)))
    });
  }
}
entries.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));

const canonical = Buffer.concat(
  entries.map((e) =>
    Buffer.concat([
      Buffer.from(e.path),
      Buffer.from([0]),
      Buffer.from(e.hex),
      Buffer.from('\n')
    ])
  )
);
const content = sha(canonical);

const commit = execSync('git rev-parse HEAD', { cwd: here }).toString().trim();
const approval = [
  `slug = "${slug}"`,
  'pack_version = 1',
  `content_sha256 = "${content}"`,
  `public_commit = "${commit}"`,
  'approved_by = "test-harness"',
  'approved_at = "2026-11-18"',
  ''
].join('\n');

const target = join(scratchRoot, 'packs', slug, 'approval.toml');
writeFileSync(target, approval);
console.log(`stamped ${target}`);
console.log(`content_sha256 ${content}`);
