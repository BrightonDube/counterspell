import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import Beasties from 'beasties';

const build = resolve('build');
const index = resolve(build, 'index.html');
const html = await readFile(index, 'utf8');
const beasties = new Beasties({
  path: build,
  publicPath: '/',
  preload: 'swap',
  noscriptFallback: true,
  logLevel: 'warn'
});

await writeFile(index, await beasties.process(html));
