// test-change: R14.2 (T48 CST -> GuessNode mapping on real source)
//
// Parses real Python and JavaScript with the shipping grammars and asserts the
// worker's mapper feeds `analyze` the shapes its tests build by hand. This is
// the seam where grammar node names and field names could silently diverge.

import { join } from 'node:path';
import { Language, Parser } from 'web-tree-sitter';
import { beforeAll, describe, expect, it } from 'vitest';

import { mapNode } from './cst';
import { analyze } from './heuristics';

const NODE_MODULES = join(import.meta.dirname, '../../../../node_modules');

const parsers = new Map<string, Parser>();

async function guessFor(
  language: 'python' | 'javascript',
  code: string
): Promise<ReturnType<typeof analyze>> {
  let parser = parsers.get(language);
  if (!parser) {
    parser = new Parser();
    parser.setLanguage(
      await Language.load(
        join(
          NODE_MODULES,
          `tree-sitter-${language}/tree-sitter-${language}.wasm`
        )
      )
    );
    parsers.set(language, parser);
  }
  const tree = parser.parse(code);
  if (!tree) throw new Error('parse returned no tree');
  return analyze(mapNode(tree.rootNode));
}

beforeAll(async () => {
  await Parser.init({
    locateFile: () => join(NODE_MODULES, 'web-tree-sitter/web-tree-sitter.wasm')
  });
});

describe('cst mapping: python', () => {
  it('reads a linear loop as n', async () => {
    const guess = await guessFor(
      'python',
      'def f(nums):\n    for x in nums:\n        print(x)\n'
    );
    expect(guess.guess_class).toBe('n');
  });

  it('reads nested loops as n^2', async () => {
    const guess = await guessFor(
      'python',
      'def f(nums):\n    for i in range(len(nums)):\n        for j in range(len(nums)):\n            print(i, j)\n'
    );
    expect(guess.guess_class).toBe('n^2');
  });

  it('reads a halving while loop as log n', async () => {
    const guess = await guessFor(
      'python',
      'def f(n):\n    while n > 0:\n        n = n // 2\n'
    );
    expect(guess.guess_class).toBe('log n');
  });

  it('reads sort then a pass as n log n', async () => {
    const guess = await guessFor(
      'python',
      'def f(nums):\n    nums.sort()\n    for x in nums:\n        print(x)\n'
    );
    expect(guess.guess_class).toBe('n log n');
  });

  it('reads unmemoised double recursion as 2^n', async () => {
    const guess = await guessFor(
      'python',
      'def fib(n):\n    if n < 2:\n        return n\n    return fib(n - 1) + fib(n - 2)\n'
    );
    expect(guess.guess_class).toBe('2^n');
  });

  it('honours a memo decorator', async () => {
    const guess = await guessFor(
      'python',
      'from functools import cache\n@cache\ndef fib(n):\n    if n < 2:\n        return n\n    return fib(n - 1) + fib(n - 2)\n'
    );
    expect(guess.guess_class).not.toBe('2^n');
  });
});

describe('cst mapping: javascript', () => {
  it('reads a for..of loop as n', async () => {
    const guess = await guessFor(
      'javascript',
      'function f(nums) {\n  for (const x of nums) {\n    console.log(x);\n  }\n}\n'
    );
    expect(guess.guess_class).toBe('n');
  });

  it('reads a C-style for loop as n', async () => {
    const guess = await guessFor(
      'javascript',
      'function f(nums) {\n  for (let i = 0; i < nums.length; i++) {\n    console.log(nums[i]);\n  }\n}\n'
    );
    expect(guess.guess_class).toBe('n');
  });

  it('reads an arrow function bound to a name', async () => {
    const guess = await guessFor(
      'javascript',
      'const fib = (n) => {\n  if (n < 2) return n;\n  return fib(n - 1) + fib(n - 2);\n};\n'
    );
    expect(guess.guess_class).toBe('2^n');
  });

  it('reads a halving while loop as log n', async () => {
    const guess = await guessFor(
      'javascript',
      'function f(n) {\n  while (n > 0) {\n    n = n / 2;\n  }\n}\n'
    );
    expect(guess.guess_class).toBe('log n');
  });

  it('reads sort then a pass as n log n', async () => {
    const guess = await guessFor(
      'javascript',
      'function f(nums) {\n  nums.sort();\n  for (const x of nums) {\n    console.log(x);\n  }\n}\n'
    );
    expect(guess.guess_class).toBe('n log n');
  });

  it('does not read `in` on a Set as a linear step', async () => {
    const guess = await guessFor(
      'javascript',
      'function f(nums) {\n  const seen = new Set();\n  for (const x of nums) {\n    if (x in seen) seen.add(x);\n  }\n}\n'
    );
    expect(guess.guess_class).not.toBe('n^2');
  });

  it('reads indexOf inside a loop as a linear step', async () => {
    const guess = await guessFor(
      'javascript',
      'function f(nums, t) {\n  for (const x of nums) {\n    if (nums.indexOf(x) === t) return x;\n  }\n}\n'
    );
    expect(guess.guess_class).toBe('n^2');
  });

  it('keeps nesting in unbraced loops and branches', async () => {
    const guess = await guessFor(
      'javascript',
      'function f(nums) {\n  for (const x of nums)\n    for (const y of nums)\n      if (x === y) console.log(x);\n}\n'
    );
    expect(guess.guess_class).toBe('n^2');
  });

  it('keeps signals inside parentheses', async () => {
    const halving = await guessFor(
      'javascript',
      'function f(n) {\n  while (n > 1) {\n    n = (n - (n % 2)) / 2;\n  }\n}\n'
    );
    expect(halving.guess_class).toBe('log n');
    const recursion = await guessFor(
      'javascript',
      'function fib(n) {\n  return (fib(n - 1)) + (fib(n - 2));\n}\n'
    );
    expect(recursion.guess_class).toBe('2^n');
  });
});
