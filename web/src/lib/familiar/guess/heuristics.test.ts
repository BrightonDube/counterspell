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

// while n > 1:
//     half = n // 4
//     n = half * 2   (quarter scaled back up still halves n)
const SCALED_QUARTER = fn(
  'scaled_quarter',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit('1')]), [
      assign('half', op('//', [id('n'), lit('4')])),
      assign('n', op('*', [id('half'), lit('2')]))
    ])
  ]
);

// while n > 1:
//     q = n // 4 + n // 4
//     n = q * 2   (two quarters summed then doubled restores n)
const SUMMED_QUARTERS = fn(
  'summed_quarters',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit('1')]), [
      assign(
        'q',
        op('+', [op('//', [id('n'), lit('4')]), op('//', [id('n'), lit('4')])])
      ),
      assign('n', op('*', [id('q'), lit('2')]))
    ])
  ]
);

// while n > 1:
//     k = 1
//     half = (n // 2) // k
//     n = half * 2 - 1   (unknown divisor, doubling restores n)
const NESTED_UNKNOWN_DIVISOR = fn(
  'nested_unknown_div',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit('1')]), [
      assign('k', lit('1')),
      assign('half', op('//', [op('//', [id('n'), lit('2')]), id('k')])),
      assign('n', op('-', [op('*', [id('half'), lit('2')]), lit('1')]))
    ])
  ]
);

// while n > 1:
//     half = n - n // 2
//     n = half   (n minus its own half is still half)
const COMPLEMENT_HALF = fn(
  'complement_half',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit('1')]), [
      assign('half', op('-', [id('n'), op('//', [id('n'), lit('2')])])),
      assign('n', id('half'))
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

// while lo < hi:
//     mid = lo + (hi - lo) // 2   (midpoint written with an offset)
//     if a[mid] < t: lo = mid + 1
//     else:          hi = mid
const OFFSET_MIDPOINT_SEARCH = fn(
  'bsearch3',
  ['a', 't'],
  [
    whileLoop(cmp('<', [id('lo'), id('hi')]), [
      assign(
        'mid',
        op('+', [id('lo'), op('//', [op('-', [id('hi'), id('lo')]), lit()])])
      ),
      ifNode(
        cmp('<', [call('get', [id('a'), id('mid')]), id('t')]),
        [assign('lo', op('+', [id('mid'), lit()]))],
        [assign('hi', id('mid'))]
      )
    ])
  ]
);

// while n > 0:
//     previous = n
//     n = previous // 2
//     n = previous - 1   (last update wins — net effect is a countdown)
const OVERWRITTEN_HALVING = fn(
  'fake_halving',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit()]), [
      assign('previous', id('n')),
      assign('n', op('//', [id('previous'), lit()])),
      assign('n', op('-', [id('previous'), lit()]))
    ])
  ]
);

describe('cost model edge cases', () => {
  it('reads a midpoint written as lo + (hi - lo) // 2 as log n', () => {
    expect(analyze(OFFSET_MIDPOINT_SEARCH).guess_class).toBe('log n');
  });

  it('lets a later countdown update override an earlier halving', () => {
    expect(analyze(OVERWRITTEN_HALVING).guess_class).toBe('n');
  });
});

// while n > 1:
//     n = (n // 2) * 2 - 1   (the multiplication cancels the division)
const CANCELLED_HALVING = fn(
  'cancelled',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit()]), [
      assign(
        'n',
        op('-', [op('*', [op('//', [id('n'), lit()]), lit()]), lit()])
      )
    ])
  ]
);

// while n > 1:
//     half = n // 2
//     n = n - half         (n - n/2 = n/2 — a true geometric shrink)
const SUBTRACTED_HALF = fn(
  'subtract_half',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit()]), [
      assign('half', op('//', [id('n'), lit()])),
      assign('n', op('-', [id('n'), id('half')]))
    ])
  ]
);

// while n > 1:
//     n = n // 2
//     n = n - 1            (net effect per pass is n - 1 — linear)
const HALVING_THEN_DECREMENT = fn(
  'halve_then_dec',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit()]), [
      assign('n', op('//', [id('n'), lit()])),
      assign('n', op('-', [id('n'), lit()]))
    ])
  ]
);

describe('cost model edge cases', () => {
  it('does not read a division cancelled by multiplication as halving', () => {
    expect(analyze(CANCELLED_HALVING).guess_class).toBe('n');
  });

  it('reads n = n - half as a geometric shrink', () => {
    expect(analyze(SUBTRACTED_HALF).guess_class).toBe('log n');
  });

  it('reads a halving overwritten by a decrement as linear', () => {
    expect(analyze(HALVING_THEN_DECREMENT).guess_class).toBe('n');
  });
});

// while n > 1:
//     half = n // 2
//     n = n - (half % 2) - 1   (only a parity peek — still a countdown)
const PARITY_DECREMENT = fn(
  'parity_step',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit()]), [
      assign('half', op('//', [id('n'), lit('2')])),
      assign(
        'n',
        op('-', [op('-', [id('n'), op('%', [id('half'), lit('2')])]), lit('1')])
      )
    ])
  ]
);

// while n > 1: n = (n * 2) // 4    (net shrink 1/2 — genuinely logarithmic)
const DOUBLE_THEN_QUARTER = fn(
  'double_quarter',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit('1')]), [
      assign('n', op('//', [op('*', [id('n'), lit('2')]), lit('4')]))
    ])
  ]
);

// while n > 1: n = (n // 4) * 2    (net shrink 1/2 — genuinely logarithmic)
const QUARTER_THEN_DOUBLE = fn(
  'quarter_double',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit('1')]), [
      assign('n', op('*', [op('//', [id('n'), lit('4')]), lit('2')]))
    ])
  ]
);

describe('cost model edge cases', () => {
  it('does not read a parity check on a halved value as halving', () => {
    expect(analyze(PARITY_DECREMENT).guess_class).toBe('n');
  });

  it('reads (n * 2) // 4 as a geometric shrink', () => {
    expect(analyze(DOUBLE_THEN_QUARTER).guess_class).toBe('log n');
  });

  it('reads (n // 4) * 2 as a geometric shrink', () => {
    expect(analyze(QUARTER_THEN_DOUBLE).guess_class).toBe('log n');
  });
});

// while n > 1: n = n >> 1   (right shift by one halves n)
const SHIFT_HALVING = fn(
  'shift_halve',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit('1')]), [
      assign('n', op('>>', [id('n'), lit('1')]))
    ])
  ]
);

// while n > 3: n = (n // 2) * n   (bound grows quadratically, not halves)
const INFLATED_BOUND = fn(
  'inflated',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit('3')]), [
      assign('n', op('*', [op('//', [id('n'), lit('2')]), id('n')]))
    ])
  ]
);

// while n > 1:
//     half = n // 2
//     n = n - half * 1   (half times one is still half — geometric shrink)
const IDENTITY_SCALED_HALF = fn(
  'scaled_half',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit('1')]), [
      assign('half', op('//', [id('n'), lit('2')])),
      assign('n', op('-', [id('n'), op('*', [id('half'), lit('1')])]))
    ])
  ]
);

// while n > 1:
//     half = n // 2
//     n = n + half       (bound grows by half — no shrink at all)
const GROWING_BOUND = fn(
  'grow_half',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit('1')]), [
      assign('half', op('//', [id('n'), lit('2')])),
      assign('n', op('+', [id('n'), id('half')]))
    ])
  ]
);

describe('cost model edge cases', () => {
  it('reads a right-shift loop as log n', () => {
    expect(analyze(SHIFT_HALVING).guess_class).toBe('log n');
  });

  it('does not read (n // 2) * n as a shrink', () => {
    expect(analyze(INFLATED_BOUND).guess_class).toBe('n');
  });

  it('reads n - half * 1 as a geometric shrink', () => {
    expect(analyze(IDENTITY_SCALED_HALF).guess_class).toBe('log n');
  });

  it('does not read n + half as a shrink', () => {
    expect(analyze(GROWING_BOUND).guess_class).toBe('n');
  });
});

// while n > 1:
//     half = n // 2
//     n = half * 2 - 1   (doubling the half restores n -> linear countdown)
const DOUBLED_HALF = fn(
  'doubled_half',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit('1')]), [
      assign('half', op('//', [id('n'), lit('2')])),
      assign('n', op('-', [op('*', [id('half'), lit('2')]), lit('1')]))
    ])
  ]
);

// while i < n:
//     half = (n - i) // 2
//     i = i + half + 1   (each step halves the remaining distance)
const HALF_DISTANCE = fn(
  'half_distance',
  ['i', 'n'],
  [
    whileLoop(cmp('<', [id('i'), id('n')]), [
      assign('half', op('//', [op('-', [id('n'), id('i')]), lit('2')])),
      assign('i', op('+', [op('+', [id('i'), id('half')]), lit('1')]))
    ])
  ]
);

// while n > 1:
//     k = 1
//     n = ((n // k) * 2) // 2 - 1   (unknown divisor, then *2/2 -> countdown)
const UNKNOWN_DIVISOR = fn(
  'unknown_divisor',
  ['n'],
  [
    whileLoop(cmp('>', [id('n'), lit('1')]), [
      assign('k', lit('1')),
      assign(
        'n',
        op('-', [
          op('//', [
            op('*', [op('//', [id('n'), id('k')]), lit('2')]),
            lit('2')
          ]),
          lit('1')
        ])
      )
    ])
  ]
);

describe('cost model edge cases', () => {
  it('reads a right-shift loop as log n', () => {
    expect(analyze(SHIFT_HALVING).guess_class).toBe('log n');
  });

  it('does not read (n // 2) * n as a shrink', () => {
    expect(analyze(INFLATED_BOUND).guess_class).toBe('n');
  });

  it('reads n - half * 1 as a geometric shrink', () => {
    expect(analyze(IDENTITY_SCALED_HALF).guess_class).toBe('log n');
  });

  it('does not read n + half as a shrink', () => {
    expect(analyze(GROWING_BOUND).guess_class).toBe('n');
  });

  it('does not read half * 2 - 1 as a shrink', () => {
    expect(analyze(DOUBLED_HALF).guess_class).toBe('n');
  });

  it('reads i + (n - i) // 2 as a halving of the remaining distance', () => {
    expect(analyze(HALF_DISTANCE).guess_class).toBe('log n');
  });

  it('does not read an unknown-name divisor as a shrink', () => {
    expect(analyze(UNKNOWN_DIVISOR).guess_class).toBe('n');
  });
  it('reads half * 2 of a quarter as a geometric shrink', () => {
    expect(analyze(SCALED_QUARTER).guess_class).toBe('log n');
  });
  it('does not read summed quarters doubled as a shrink', () => {
    expect(analyze(SUMMED_QUARTERS).guess_class).toBe('n');
  });

  it('does not read a nested unknown divisor scaled back as a shrink', () => {
    expect(analyze(NESTED_UNKNOWN_DIVISOR).guess_class).toBe('n');
  });
  it('reads n - n // 2 as a halving', () => {
    expect(analyze(COMPLEMENT_HALF).guess_class).toBe('log n');
  });
});
