// test-change: R14.2 (T47 pure heuristics over a small syntax-tree interface)

import { describe, expect, it } from 'vitest';

import { analyze, type GuessNode } from './heuristics';

// Small builders for the plain-object syntax trees the worker will produce.
const id = (text: string): GuessNode => ({ type: 'name', text, children: [] });
const lit = (text = ''): GuessNode => ({ type: 'literal', text, children: [] });
const call = (name: string, args: GuessNode[] = []): GuessNode => ({
  type: 'call',
  text: name,
  children: args
});
const mcall = (
  recv: GuessNode,
  name: string,
  args: GuessNode[] = []
): GuessNode => ({
  type: 'member_call',
  text: name,
  children: [recv, ...args]
});
const forLoop = (iterable: GuessNode, body: GuessNode[]): GuessNode => ({
  type: 'for',
  text: '',
  children: [iterable, ...body]
});
const whileLoop = (cond: GuessNode, body: GuessNode[]): GuessNode => ({
  type: 'while',
  text: '',
  children: [cond, ...body]
});
const fn = (
  name: string,
  params: string[],
  body: GuessNode[],
  decorators: string[] = []
): GuessNode => ({
  type: 'function',
  text: name,
  children: [
    { type: 'params', text: '', children: params.map(id) },
    ...decorators.map((d) => ({ type: 'decorator', text: d, children: [] })),
    ...body
  ]
});
const assign = (name: string, value: GuessNode): GuessNode => ({
  type: 'assign',
  text: name,
  children: [value]
});
const op = (operator: string, children: GuessNode[]): GuessNode => ({
  type: 'binary_op',
  text: operator,
  children
});
const cmp = (operator: string, children: GuessNode[]): GuessNode => ({
  type: 'compare',
  text: operator,
  children
});

// Golden snippets: the common code shapes the guess must recognise.

// for i in range(len(nums)):
//     for j in range(len(nums)):
//         print(i, j)
const DOUBLE_LOOP = fn(
  'pairs',
  ['nums'],
  [
    forLoop(call('range', [call('len', [id('nums')])]), [
      forLoop(call('range', [call('len', [id('nums')])]), [
        call('print', [id('i'), id('j')])
      ])
    ])
  ]
);

// seen = set()
// for x in nums:
//     if x not in seen: seen.add(x)
const HASH_MAP_SINGLE_PASS = fn(
  'dedupe',
  ['nums'],
  [
    assign('seen', call('set')),
    forLoop(id('nums'), [
      cmp('in', [id('x'), id('seen')]),
      mcall(id('seen'), 'add', [id('x')])
    ])
  ]
);

// nums.sort()
// for i in range(len(nums) - 1):
//     best = min(best, nums[i + 1] - nums[i])
const SORT_THEN_SCAN = fn(
  'closest',
  ['nums'],
  [
    mcall(id('nums'), 'sort'),
    forLoop(call('range', [op('-', [call('len', [id('nums')]), lit()])]), [
      call('min', [id('best'), op('-', [id('nums'), id('nums')])])
    ])
  ]
);

// while lo < hi:
//     mid = (lo + hi) // 2
//     if nums[mid] < t: lo = mid + 1
//     else: hi = mid
const BINARY_SEARCH = fn(
  'find',
  ['nums', 't'],
  [
    assign('lo', lit()),
    assign('hi', call('len', [id('nums')])),
    whileLoop(cmp('<', [id('lo'), id('hi')]), [
      assign('mid', op('//', [op('+', [id('lo'), id('hi')]), lit()])),
      assign('lo', op('+', [id('mid'), lit()])),
      assign('hi', id('mid'))
    ])
  ]
);

// def fib(n):
//     if n < 2: return n
//     return fib(n - 1) + fib(n - 2)
const NAIVE_FIBONACCI = fn(
  'fib',
  ['n'],
  [
    {
      type: 'return',
      text: '',
      children: [op('+', [call('fib', [id('n')]), call('fib', [id('n')])])]
    }
  ]
);

const MEMOIZED_FIBONACCI = fn(
  'fib',
  ['n'],
  [
    {
      type: 'return',
      text: '',
      children: [op('+', [call('fib', [id('n')]), call('fib', [id('n')])])]
    }
  ],
  ['cache']
);

describe('golden snippets', () => {
  it('reads a double loop over an input parameter as n^2', () => {
    const guess = analyze(DOUBLE_LOOP);
    expect(guess.guess_class).toBe('n^2');
    expect(guess.confidence).toBe('med');
    expect(guess.reasons.length).toBeGreaterThan(0);
  });

  it('reads a hash-map single pass as n', () => {
    const guess = analyze(HASH_MAP_SINGLE_PASS);
    expect(guess.guess_class).toBe('n');
    expect(guess.confidence).toBe('med');
  });

  it('reads sort then a single pass as n log n', () => {
    const guess = analyze(SORT_THEN_SCAN);
    expect(guess.guess_class).toBe('n log n');
    expect(guess.confidence).toBe('med');
  });

  it('reads a halving loop as log n', () => {
    const guess = analyze(BINARY_SEARCH);
    expect(guess.guess_class).toBe('log n');
    expect(guess.confidence).toBe('med');
  });

  it('reads multi-self-call recursion without memo as 2^n', () => {
    const guess = analyze(NAIVE_FIBONACCI);
    expect(guess.guess_class).toBe('2^n');
    expect(guess.confidence).toBe('med');
  });

  it('does not fire on memoized recursion', () => {
    const guess = analyze(MEMOIZED_FIBONACCI);
    expect(guess.guess_class).not.toBe('2^n');
  });
});

describe('guard rails', () => {
  it('returns non-empty reasons whenever a class is returned', () => {
    for (const tree of [
      DOUBLE_LOOP,
      HASH_MAP_SINGLE_PASS,
      SORT_THEN_SCAN,
      BINARY_SEARCH,
      NAIVE_FIBONACCI
    ]) {
      const guess = analyze(tree);
      if (guess.guess_class !== null) {
        expect(guess.reasons.length).toBeGreaterThan(0);
      }
    }
  });

  it('never returns confidence above med', () => {
    for (const tree of [
      DOUBLE_LOOP,
      HASH_MAP_SINGLE_PASS,
      SORT_THEN_SCAN,
      BINARY_SEARCH,
      NAIVE_FIBONACCI,
      MEMOIZED_FIBONACCI
    ]) {
      expect(['low', 'med']).toContain(analyze(tree).confidence);
    }
  });

  it('is deterministic', () => {
    expect(analyze(NAIVE_FIBONACCI)).toEqual(analyze(NAIVE_FIBONACCI));
  });

  it('drops to low when an unknown call sits inside a loop', () => {
    const tree = fn(
      'mystery',
      ['nums'],
      [forLoop(id('nums'), [call('external_lookup', [id('x')])])]
    );
    const guess = analyze(tree);
    expect(guess.guess_class).toBe('n');
    expect(guess.confidence).toBe('low');
  });

  it('returns no class when no signal fires', () => {
    const guess = analyze(fn('noop', [], [call('print', [lit()])]));
    expect(guess.guess_class).toBeNull();
    expect(guess.confidence).toBe('low');
  });
});

// Regression shapes: each pass costs independently, halving beats linear.

// while n > 1: n = n // 2
const DIRECT_HALVING = fn(
  'halve',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit()]), [
      assign('n', op('//', [id('n'), lit()]))
    ])
  ]
);

// for x in nums: nums.sort()
const SORT_INSIDE_LOOP = fn(
  'resort',
  ['nums'],
  [forLoop(id('nums'), [mcall(id('nums'), 'sort')])]
);

// seen = []; for x in nums: if x in seen: seen.append(x)
// plus an unrelated double loop
const LINEAR_SEARCH_PASS_AND_DOUBLE_LOOP = fn(
  'two_passes',
  ['nums'],
  [
    assign('seen', { type: 'literal', text: '[]', children: [] }),
    forLoop(id('nums'), [
      cmp('in', [id('x'), id('seen')]),
      mcall(id('seen'), 'append', [id('x')])
    ]),
    forLoop(id('nums'), [
      forLoop(id('nums'), [call('print', [id('x'), id('y')])])
    ])
  ]
);

describe('cost model', () => {
  it('reads a direct halving loop as log n, not n', () => {
    const guess = analyze(DIRECT_HALVING);
    expect(guess.guess_class).toBe('log n');
  });

  it('does not read a sort inside a loop as a one-off preprocessing pass', () => {
    const guess = analyze(SORT_INSIDE_LOOP);
    expect(guess.guess_class).not.toBe('n log n');
  });

  it('costs separate passes independently instead of adding levels', () => {
    const guess = analyze(LINEAR_SEARCH_PASS_AND_DOUBLE_LOOP);
    expect(guess.guess_class).toBe('n^2');
  });

  it('treats membership on a list literal as a scan, not a hash lookup', () => {
    const guess = analyze(LINEAR_SEARCH_PASS_AND_DOUBLE_LOOP);
    expect(guess.reasons.join(' ')).toContain('linear');
  });
});

// for x in sorted(nums): print(x)
const SORT_IN_ITERABLE = fn(
  'scan_sorted',
  ['nums'],
  [forLoop(call('sorted', [id('nums')]), [call('print', [id('x')])])]
);

// while n > 0:
//     half = n // 2   (unrelated division — bound shrinks by 1)
//     n = n - 1
const LINEAR_COUNTDOWN = fn(
  'countdown',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit()]), [
      assign('half', op('//', [id('n'), lit()])),
      assign('n', op('-', [id('n'), lit()]))
    ])
  ]
);

// while n > 1:
//     external(n)      (unknown call inside the loop)
//     n = n // 2
const HALVING_WITH_UNKNOWN_CALL = fn(
  'halve_external',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit()]), [
      call('external', [id('n')]),
      assign('n', op('//', [id('n'), lit()]))
    ])
  ]
);

describe('cost model edge cases', () => {
  it('reads a sort in the iterable as preprocessing: n log n', () => {
    expect(analyze(SORT_IN_ITERABLE).guess_class).toBe('n log n');
  });

  it('does not read an unrelated division as a halving loop', () => {
    expect(analyze(LINEAR_COUNTDOWN).guess_class).toBe('n');
  });

  it('drops to low when an unknown call sits inside a halving loop', () => {
    const guess = analyze(HALVING_WITH_UNKNOWN_CALL);
    expect(guess.guess_class).toBe('log n');
    expect(guess.confidence).toBe('low');
  });
});

const ifNode = (
  cond: GuessNode,
  then: GuessNode[],
  otherwise: GuessNode[] = []
): GuessNode => ({
  type: 'if',
  text: '',
  children: [cond, ...then, ...otherwise]
});

// while lo < hi:
//     mid = (lo + hi) // 2
//     if a[mid] < t: lo = mid + 1
//     else:          hi = mid
const NESTED_BINARY_SEARCH = fn(
  'bsearch2',
  ['a', 't'],
  [
    assign('lo', lit()),
    assign('hi', lit()),
    whileLoop(cmp('<', [id('lo'), id('hi')]), [
      assign('mid', op('//', [op('+', [id('lo'), id('hi')]), lit()])),
      ifNode(
        cmp('<', [call('get', [id('a'), id('mid')]), id('t')]),
        [assign('lo', op('+', [id('mid'), lit()]))],
        [assign('hi', id('mid'))]
      )
    ])
  ]
);

// while n > 0:
//     n = n - (10 // 2)   (constant decrement, not a geometric shrink)
const CONSTANT_DIVIDE_DECREMENT = fn(
  'step_down',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit()]), [
      assign('n', op('-', [id('n'), op('//', [lit(), lit()])]))
    ])
  ]
);

describe('cost model edge cases', () => {
  it('reads a binary search whose bound updates sit inside if/else', () => {
    expect(analyze(NESTED_BINARY_SEARCH).guess_class).toBe('log n');
  });

  it('does not read a constant division in the decrement as halving', () => {
    expect(analyze(CONSTANT_DIVIDE_DECREMENT).guess_class).toBe('n');
  });
});

// while n > 0:
//     def helper():
//         n = n // 2     (uncalled helper — not this loop's update)
//     n = n - 1
const NESTED_FN_DIVISION = fn(
  'countdown2',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit()]), [
      fn('helper', [], [assign('n', op('//', [id('n'), lit()]))]),
      assign('n', op('-', [id('n'), lit()]))
    ])
  ]
);

// while n > 0:
//     previous = n
//     n = previous // 2   (halving through a temporary copy)
const ALIASED_HALVING = fn(
  'halve_alias',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit()]), [
      assign('previous', id('n')),
      assign('n', op('//', [id('previous'), lit()]))
    ])
  ]
);

describe('cost model edge cases', () => {
  it('ignores an uncalled nested helper when pricing the loop', () => {
    expect(analyze(NESTED_FN_DIVISION).guess_class).toBe('n');
  });

  it('reads a halving applied through a temporary copy', () => {
    expect(analyze(ALIASED_HALVING).guess_class).toBe('log n');
  });
});
