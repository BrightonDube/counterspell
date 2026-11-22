// Pure live-typing guess (efficiency-familiar T47, R14.2).
//
// Reads a small plain-object syntax tree — the worker maps tree-sitter CST onto
// this interface — and answers a complexity guess in the measured-class
// vocabulary ("1", "log n", "n", "n log n", "n^2", "n^3", "2^n"). Confidence is
// deliberately capped at "med": this is a guess from reading the code, never a
// measured verdict.

export interface GuessNode {
  // Tree-sitter-flavoured node kind: 'function', 'params', 'decorator',
  // 'for', 'while', 'call', 'member_call', 'assign', 'compare', 'binary_op',
  // 'name', 'member', 'literal', 'return', ... unknown kinds are ignored.
  type: string;
  // The node's payload text: callee name for calls, operator for
  // compare/binary_op, target for assign, identifier for name/member.
  text: string;
  children: GuessNode[];
}

export interface Guess {
  guess_class: string | null;
  confidence: 'low' | 'med';
  reasons: string[];
}

const CLASS_RANK = new Map([
  ['1', 0],
  ['log n', 1],
  ['n', 2],
  ['n log n', 3],
  ['n^2', 4],
  ['n^3', 5],
  ['2^n', 6]
]);

// Calls the heuristics can reason about: builtins and common methods in both
// launch languages. A call outside this set inside a loop drops confidence to
// 'low' — the guess no longer knows what the loop costs.
const KNOWN_CALLS = new Set([
  'abs',
  'add',
  'all',
  'any',
  'append',
  'bool',
  'ceil',
  'chr',
  'clear',
  'contains',
  'delete',
  'dict',
  'endswith',
  'enumerate',
  'extend',
  'filter',
  'find',
  'findindex',
  'float',
  'floor',
  'get',
  'has',
  'includes',
  'index',
  'indexof',
  'insert',
  'int',
  'isdigit',
  'items',
  'join',
  'keys',
  'len',
  'list',
  'log',
  'lower',
  'lstrip',
  'map',
  'max',
  'min',
  'ord',
  'pop',
  'pow',
  'print',
  'push',
  'range',
  'remove',
  'replace',
  'reversed',
  'round',
  'rstrip',
  'set',
  'shift',
  'slice',
  'sort',
  'sorted',
  'splice',
  'split',
  'sqrt',
  'startswith',
  'str',
  'strip',
  'sum',
  'tuple',
  'unshift',
  'update',
  'upper',
  'values',
  'entries',
  'zip',
  // JavaScript spellings and builtins: the same names the C-style grammar
  // emits, so recognized calls do not drop the guess to low confidence.
  'Array',
  'BigInt',
  'Boolean',
  'Date',
  'JSON',
  'Map',
  'Math',
  'Number',
  'Object',
  'Promise',
  'Set',
  'String',
  'Symbol',
  'WeakMap',
  'WeakSet',
  'assign',
  'at',
  'charAt',
  'charCodeAt',
  'codePointAt',
  'concat',
  'copyWithin',
  'every',
  'exec',
  'fill',
  'findIndex',
  'findLast',
  'findLastIndex',
  'flat',
  'flatMap',
  'forEach',
  'freeze',
  'from',
  'includes',
  'indexOf',
  'isArray',
  'isFinite',
  'isNaN',
  'lastIndexOf',
  'localeCompare',
  'match',
  'matchAll',
  'now',
  'of',
  'padEnd',
  'padStart',
  'parse',
  'parseFloat',
  'parseInt',
  'random',
  'reduce',
  'reduceRight',
  'repeat',
  'replaceAll',
  'reverse',
  'search',
  'sign',
  'some',
  'stringify',
  'substring',
  'test',
  'toReversed',
  'toSorted',
  'toSpliced',
  'toString',
  'trim',
  'trimEnd',
  'trimStart',
  'trunc',
  'valueOf',
  'with'
]);

// Methods that make an in-loop step linear in the container (linear search and
// front-of-list mutation).
const LINEAR_METHODS = new Set([
  'contains',
  'find',
  'findindex',
  'indexOf',
  'findIndex',
  'lastIndexOf',
  'includes',
  'index',
  'indexof',
  'insert',
  'remove',
  'shift',
  'splice',
  'unshift'
]);

const SORT_CALLS = new Set(['sort', 'sorted', 'sort_by', 'sortby', 'toSorted']);
const MEMO_DECORATORS = new Set([
  'cache',
  'lru_cache',
  'memoize',
  'memo',
  'cached'
]);
const MEMO_NAME = /^_?(memo|dp|cache|cached|table|seen|lookup|store)/i;
const HASH_SOURCES = new Set([
  'set',
  'dict',
  'Set',
  'Map',
  'WeakMap',
  'WeakSet'
]);

interface FnReport {
  name: string;
  classes: Map<string, string>; // class label -> reason
  signalCount: number;
  unknownInLoop: boolean;
}

// Subtree helpers -----------------------------------------------------------

function containsParamName(node: GuessNode, params: Set<string>): boolean {
  if (
    (node.type === 'name' || node.type === 'member') &&
    params.has(node.text)
  ) {
    return true;
  }
  return node.children.some((child) => containsParamName(child, params));
}

function callsIn(node: GuessNode, out: GuessNode[] = []): GuessNode[] {
  if (node.type === 'call' || node.type === 'member_call') {
    out.push(node);
  }
  for (const child of node.children) {
    callsIn(child, out);
  }
  return out;
}

function namesIn(node: GuessNode, out: Set<string>): Set<string> {
  if (node.type === 'name' || node.type === 'member') out.add(node.text);
  for (const child of node.children) namesIn(child, out);
  return out;
}

function assignsIn(node: GuessNode, out: GuessNode[] = []): GuessNode[] {
  if (node.type === 'function') return out; // an uncalled helper is not an update
  if (node.type === 'assign') out.push(node);
  for (const child of node.children) assignsIn(child, out);
  return out;
}

// Numeric magnitude of a literal; unparseable literals assume the common
// halving constant 2.
function numeric(node: GuessNode): number {
  const v = Number(node.text);
  return Number.isFinite(v) && v > 0 ? v : 2;
}

// The coefficient a subexpression applies to the target-named bound it
// derives from: `*` multiplies by its literal (two bound-derived terms make
// the product non-linear — no coefficient), `//` `/` divide by their
// literal, `>>` divides by 2^shift, `%` and calls discard the bound, and
// `+`/`-` keep the dominant bound term. Unknown names carry Infinity so a
// param like `k` can never manufacture a shrink.
function boundFactor(node: GuessNode, targets: Set<string>): number {
  if (node.type === 'literal') return 0;
  if (node.type === 'name' || node.type === 'member') {
    return targets.has(node.text) ? 1 : Infinity;
  }
  if (node.type === 'call' || node.type === 'member_call') return Infinity;
  if (node.type !== 'binary_op') {
    return Math.max(
      0,
      ...node.children.map((child) => boundFactor(child, targets))
    );
  }
  if (node.text === '*') {
    const boundChildren = node.children.filter(
      (child) => child.type !== 'literal' && boundFactor(child, targets) > 0
    );
    if (boundChildren.length > 1) return Infinity;
    return node.children.reduce(
      (acc, child) =>
        acc *
        (child.type === 'literal'
          ? numeric(child)
          : boundFactor(child, targets)),
      1
    );
  }
  if (['//', '/', '>>'].includes(node.text)) {
    const divisor = node.children.slice(1).reduce((acc, child) => {
      if (child.type === 'literal') {
        const scale =
          node.text === '>>'
            ? Math.pow(2, Math.min(numeric(child), 30))
            : numeric(child);
        return acc * scale;
      }
      return acc * boundFactor(child, targets);
    }, 1);
    return boundFactor(node.children[0], targets) / Math.max(divisor, 1);
  }
  if (node.text === '%') return 0;
  if (node.text === '-') {
    const left = boundFactor(node.children[0], targets);
    const rest = node.children.slice(1);
    const right = rest.map((child) => boundFactor(child, targets));
    // Subtracting a strict fraction of the same bound keeps a fraction:
    // `n - n // 2` is `n // 2`. Other pairs stay bound-scale — `hi - lo`
    // is a distance, and `hi - lo // 2` subtracts a fraction of a
    // different var, so the subtrahend's bound names must all appear on
    // the left.
    const leftNames = namesIn(node.children[0], new Set());
    if (
      rest.length > 0 &&
      rest.every(
        (child, i) =>
          right[i] > 0 &&
          right[i] < left &&
          [...namesIn(child, new Set())].every(
            (n) => targets.has(n) && leftNames.has(n)
          )
      )
    ) {
      return right.reduce((acc, factor) => acc - factor, left);
    }
    return Math.max(0, left, ...right);
  }
  if (node.text === '+') {
    const factors = node.children.map((child) => boundFactor(child, targets));
    if (factors.some((factor) => !Number.isFinite(factor))) return Infinity;
    const fractions = factors.filter((factor) => factor < 1);
    // Bound terms accumulate only as fractions of the bound: `n // 4 +
    // n // 4` sums to `n // 2`, while `lo + hi` stays bound-scale — its
    // dominant term keeps coefficient 1, and `lo + (hi - lo) // 2` reads
    // as a half of the remaining distance.
    if (factors.every((factor) => factor < 1)) {
      return fractions.reduce((acc, factor) => acc + factor, 0);
    }
    if (fractions.length > 0) {
      return fractions.reduce((acc, factor) => acc + factor, 0);
    }
    return Math.max(0, ...factors);
  }
  return Math.max(
    0,
    ...node.children.map((child) => boundFactor(child, targets))
  );
}

// A `//`, `/`, or `>>` by a constant counts as a bound shrink only when the
// multiplicative context leaves it a genuine fraction: `(n // 4) * 2` and
// `(n * 2) // 4` shrink (factor 1/2), `(n // 2) * 2` does not (factor 1), and
// a `//` feeding a `%` or a call result never reaches the bound. The dividend
// must name one of the target vars, so `10 // 2` and `k // 2` do not count.
function hasHalvingOpOn(
  node: GuessNode,
  targets: Set<string>,
  pathFactor = 1,
  blocked = false
): boolean {
  if (
    node.type === 'call' ||
    node.type === 'member_call' ||
    (node.type === 'binary_op' && node.text === '%')
  ) {
    return node.children.some((child) =>
      hasHalvingOpOn(child, targets, pathFactor, true)
    );
  }
  if (node.type !== 'binary_op') {
    return node.children.some((child) =>
      hasHalvingOpOn(child, targets, pathFactor, blocked)
    );
  }
  const literalScale = node.children
    .filter((child) => child.type === 'literal')
    .reduce(
      (acc, child) =>
        acc *
        (node.text === '>>'
          ? Math.pow(2, Math.min(numeric(child), 30))
          : numeric(child)),
      1
    );
  if (node.text === '*') {
    return node.children.some((child, i) =>
      hasHalvingOpOn(
        child,
        targets,
        pathFactor * literalScale,
        // A sibling mentioning any name inflates the product by an unknown
        // or bound-derived factor: `(n // 2) * n` grows.
        blocked ||
          node.children.some(
            (other, j) =>
              j !== i &&
              other.type !== 'literal' &&
              namesIn(other, new Set()).size > 0
          )
      )
    );
  }
  if (['//', '/', '>>'].includes(node.text)) {
    const namedOperands = node.children.filter(
      (operand) =>
        operand.type !== 'literal' &&
        [...namesIn(operand, new Set())].some((name) => targets.has(name))
    );
    const shrinks =
      !blocked &&
      node.children.some((c) => c.type === 'literal') &&
      namedOperands.some((operand) => {
        const factor = boundFactor(operand, targets);
        // Zero means the operand carries no bound coefficient at all
        // (`n // k` with an unknown k), not a bound shrunk to nothing.
        return factor > 0 && (pathFactor * factor) / literalScale < 1;
      });
    return (
      shrinks ||
      node.children.some((child) =>
        hasHalvingOpOn(child, targets, pathFactor / literalScale, blocked)
      )
    );
  }
  return node.children.some((child) =>
    hasHalvingOpOn(child, targets, pathFactor, blocked)
  );
}

// True when `name` appears in `node` as a subtracted operand — `(n - i)`
// subtracts `i`, so a halving var derived from it still holds a bound
// that `i` steps toward.
function appearsSubtracted(node: GuessNode, name: string): boolean {
  if (
    node.type === 'binary_op' &&
    node.text === '-' &&
    node.children
      .slice(1)
      .some(
        (child) =>
          (child.type === 'name' || child.type === 'member') &&
          child.text === name
      )
  ) {
    return true;
  }
  return node.children.some((child) => appearsSubtracted(child, name));
}

// True when `name` is referenced in `node` where the bound still flows
// through it — under `+`/`-`/bare positions, or a `*` whose literals keep
// the total coefficient below one (`nameFactor` is the fraction the
// halving var's own definition carried, so `n // 4` scaled back by `* 2`
// still halves while `n // 2 * 2` restores the bound). Inside `%`, `/`,
// `//`, `>>`, or a call the reference proves nothing (`half % 2` is a
// parity peek). When the target's own name appears in the value, the
// reference must sit under a `-` (`n - half` shrinks, `n + half` grows)
// unless the halving var itself was computed from the target subtracted
// out of the bound — `half = (n - i) // 2` makes `i = i + half` halve the
// remaining distance.
function referencesBarely(
  node: GuessNode,
  name: string,
  targetInValue: boolean,
  allowPlus: boolean,
  nameFactor: number,
  underMinus = false,
  blocked = false
): boolean {
  if (
    node.type === 'call' ||
    node.type === 'member_call' ||
    (node.type === 'binary_op' && ['%', '/', '//', '>>'].includes(node.text))
  ) {
    return node.children.some((child) =>
      referencesBarely(
        child,
        name,
        targetInValue,
        allowPlus,
        nameFactor,
        underMinus,
        true
      )
    );
  }
  if (node.type === 'binary_op' && node.text === '*') {
    const literalProduct = node.children
      .filter((child) => child.type === 'literal')
      .reduce((acc, child) => acc * numeric(child), 1);
    const transparent =
      nameFactor > 0 &&
      nameFactor * literalProduct < 1 &&
      node.children.every(
        (child) =>
          child.type === 'literal' ||
          ((child.type === 'name' || child.type === 'member') &&
            child.text === name)
      );
    return node.children.some((child) =>
      referencesBarely(
        child,
        name,
        targetInValue,
        allowPlus,
        nameFactor,
        underMinus,
        blocked || !transparent
      )
    );
  }
  const seenMinus =
    underMinus || (node.type === 'binary_op' && node.text === '-');
  if (
    !blocked &&
    (node.type === 'name' || node.type === 'member') &&
    node.text === name
  ) {
    return (
      nameFactor > 0 &&
      nameFactor < 1 &&
      (!targetInValue || seenMinus || allowPlus)
    );
  }
  return node.children.some((child) =>
    referencesBarely(
      child,
      name,
      targetInValue,
      allowPlus,
      nameFactor,
      seenMinus,
      blocked
    )
  );
}

// True when a `while` body shrinks a variable the condition tests by a
// geometric factor — `x = x // 2`, `x >>= 1`, `previous = n; n = previous // 2`
// — or the binary-search shape where a bound var takes `mid` and `mid` came
// from a halving, including updates nested inside `if`/`else`. An unrelated
// division (`half = n // 2; n = n - 1`) or a constant decrement
// (`n = n - (10 // 2)`) is not a halving loop.
function halvesBound(cond: GuessNode | undefined, body: GuessNode[]): boolean {
  if (!cond) return false;
  const condNames = namesIn(cond, new Set());
  const assigns = body.flatMap((node) => assignsIn(node));
  // Condition vars plus anything a copy points back at: `previous = n` makes
  // `previous // 2` a halving of `n`.
  const derived = new Set(condNames);
  let grew = true;
  while (grew) {
    grew = false;
    for (const node of assigns) {
      if (
        !derived.has(node.text) &&
        node.children.some(
          (value) => value.type === 'name' && derived.has(value.text)
        )
      ) {
        derived.add(node.text);
        grew = true;
      }
    }
  }
  // Halving vars keep their definition so `i = i + half` can tell whether
  // `half` was computed from the target subtracted out of the bound
  // (`half = (n - i) // 2`), where adding it still halves the distance.
  const halvingValues = new Map<string, GuessNode>();
  for (const node of assigns) {
    const source = node.children.find((value) =>
      hasHalvingOpOn(value, derived)
    );
    if (source) halvingValues.set(node.text, source);
  }
  // When a condition var is written more than once per pass, only its last
  // assignment decides the step: `n = previous // 2; n = previous - 1`
  // counts down. The target's own name never satisfies the indirect rule,
  // so `n = n - 1` stays a countdown while `n = n - half` (half of n) still
  // reads as a geometric shrink.
  const lastUpdate = new Map<string, GuessNode>();
  for (const node of assigns) {
    if (condNames.has(node.text)) lastUpdate.set(node.text, node);
  }
  return [...lastUpdate.values()].some((node) =>
    node.children.some((value) => {
      const names = namesIn(value, new Set());
      const targetInValue = names.has(node.text);
      names.delete(node.text);
      return (
        hasHalvingOpOn(value, derived) ||
        [...names].some((name) => {
          const source = halvingValues.get(name);
          return (
            source !== undefined &&
            referencesBarely(
              value,
              name,
              targetInValue,
              appearsSubtracted(source, node.text),
              boundFactor(source, derived)
            )
          );
        })
      );
    })
  );
}

// One level is added when a loop over input-dependent data performs a linear
// search (membership on a list, `.index`, `includes`) or a front-of-list
// mutation (`remove`, `insert(0)`, `splice`). `in` against a known hash
// container (set/dict/Map/Set literal) is constant-time and does not count.
function isLinearInLoopOp(
  node: GuessNode,
  params: Set<string>,
  hashNames: Set<string>
): boolean {
  if (
    node.type === 'compare' &&
    (node.text === 'in' || node.text === 'not in')
  ) {
    const container = node.children[1];
    if (!container) return false;
    if (container.type === 'name' && hashNames.has(container.text))
      return false;
    return true;
  }
  if (node.type === 'member_call') {
    const receiver = node.children[0];
    if (receiver?.type === 'name' && hashNames.has(receiver.text)) return false;
    if (LINEAR_METHODS.has(node.text)) return true;
    if (node.text === 'pop' && node.children.length > 1) return true; // pop(i), not pop()
  }
  return false;
}

// Analysis ------------------------------------------------------------------

interface Walk {
  // The worst single path through the function: each counted loop level adds
  // one, and a linear step or a sort inside a counted loop adds one at that
  // depth. Separate passes never combine into a deeper class.
  maxLevel: number;
  // Set when a linear step priced into maxLevel, so the reason can say so.
  sawLinearStep: boolean;
  halving: boolean;
  sortOutsideLoop: boolean;
  unknownInLoop: boolean;
}

function walk(
  node: GuessNode,
  ctx: { params: Set<string>; hashNames: Set<string>; selfName: string },
  depth: number,
  inCountedLoop: boolean,
  inLoop: boolean,
  out: Walk
): void {
  switch (node.type) {
    case 'for': {
      const iterable = node.children[0];
      // The iterable evaluates once before iterating — a sort there is
      // preprocessing, priced in whatever loop context encloses this `for`.
      if (iterable) walk(iterable, ctx, depth, inCountedLoop, inLoop, out);
      const counted = iterable
        ? containsParamName(iterable, ctx.params)
        : false;
      const next = counted ? depth + 1 : depth;
      if (counted) out.maxLevel = Math.max(out.maxLevel, next);
      for (const child of node.children.slice(1)) {
        walk(child, ctx, next, inCountedLoop || counted, true, out);
      }
      return;
    }
    case 'while': {
      const cond = node.children[0];
      const body = node.children.slice(1);
      if (halvesBound(cond, body)) {
        // A halving loop contributes a log factor, not a linear level; it
        // never counts toward the depth even when its bound is a parameter.
        out.halving = true;
        for (const child of body) {
          walk(child, ctx, depth, inCountedLoop, true, out);
        }
        return;
      }
      const counted = cond ? containsParamName(cond, ctx.params) : false;
      const next = counted ? depth + 1 : depth;
      if (counted) out.maxLevel = Math.max(out.maxLevel, next);
      for (const child of body) {
        walk(child, ctx, next, inCountedLoop || counted, true, out);
      }
      return;
    }
    case 'function':
      return; // nested definitions are analysed separately
    case 'call':
    case 'member_call': {
      if (inLoop && !KNOWN_CALLS.has(node.text) && node.text !== ctx.selfName) {
        out.unknownInLoop = true;
      }
      if (inCountedLoop && SORT_CALLS.has(node.text)) {
        // Sorting once per iteration costs a superlinear level here.
        out.maxLevel = Math.max(out.maxLevel, depth + 1);
      } else if (!inCountedLoop && SORT_CALLS.has(node.text)) {
        out.sortOutsideLoop = true;
      }
      if (inCountedLoop && isLinearInLoopOp(node, ctx.params, ctx.hashNames)) {
        out.maxLevel = Math.max(out.maxLevel, depth + 1);
        out.sawLinearStep = true;
      }
      for (const child of node.children)
        walk(child, ctx, depth, inCountedLoop, inLoop, out);
      return;
    }
    default: {
      if (inCountedLoop && isLinearInLoopOp(node, ctx.params, ctx.hashNames)) {
        out.maxLevel = Math.max(out.maxLevel, depth + 1);
        out.sawLinearStep = true;
      }
      for (const child of node.children)
        walk(child, ctx, depth, inCountedLoop, inLoop, out);
    }
  }
}

function depthClass(depth: number): string {
  if (depth <= 0) return '1';
  if (depth === 1) return 'n';
  if (depth === 2) return 'n^2';
  return 'n^3';
}

function analyzeFunction(fn: GuessNode): FnReport {
  const paramsNode = fn.children.find((child) => child.type === 'params');
  const params = new Set(
    (paramsNode?.children ?? [])
      .filter((child) => child.type === 'name')
      .map((child) => child.text)
  );
  const body = fn.children.filter(
    (child) => child.type !== 'params' && child.type !== 'decorator'
  );
  const decorators = new Set(
    fn.children
      .filter((child) => child.type === 'decorator')
      .map((child) => child.text)
  );

  // Names bound to a hash container, so `x in seen` is not read as a scan.
  const hashNames = new Set<string>();
  for (const node of body) {
    if (node.type === 'assign') {
      const value = node.children[0];
      if (
        value &&
        ((value.type === 'literal' && value.text.startsWith('{')) ||
          (value.type === 'call' && HASH_SOURCES.has(value.text)))
      ) {
        hashNames.add(node.text);
      }
    }
  }

  const walkOut: Walk = {
    maxLevel: 0,
    sawLinearStep: false,
    halving: false,
    sortOutsideLoop: false,
    unknownInLoop: false
  };
  for (const node of body) {
    walk(
      node,
      { params, hashNames, selfName: fn.text },
      0,
      false,
      false,
      walkOut
    );
  }

  const classes = new Map<string, string>();
  let signals = 0;

  // A sort outside the loops followed by at most a single pass reads as
  // n log n and subsumes that lone pass; a sort inside a counted loop was
  // already priced into maxLevel as an extra level.
  if (walkOut.sortOutsideLoop && walkOut.maxLevel <= 1) {
    classes.set('n log n', 'sort followed by a single pass over the input');
    signals += 1;
  } else if (walkOut.maxLevel >= 1) {
    const cls = depthClass(walkOut.maxLevel);
    const reason =
      walkOut.maxLevel >= 2
        ? 'nested loops or an in-loop scan over input-dependent data'
        : 'a loop over input-dependent data';
    classes.set(
      cls,
      walkOut.sawLinearStep ? `${reason}, with a linear step inside` : reason
    );
    signals += 1;
  }

  if (walkOut.halving) {
    classes.set('log n', 'a loop that halves its bound each step');
    signals += 1;
  }

  // Two or more calls to the function's own name, with no memo marker.
  const selfCalls = body.reduce(
    (count, node) =>
      count + callsIn(node).filter((call) => call.text === fn.text).length,
    0
  );
  const memoised =
    [...decorators].some((d) => MEMO_DECORATORS.has(d)) ||
    body.some((node) => node.type === 'assign' && MEMO_NAME.test(node.text));
  if (selfCalls >= 2 && !memoised) {
    classes.set('2^n', 'two or more calls to itself with no memo');
    signals += 1;
  }

  return {
    name: fn.text,
    classes,
    signalCount: signals,
    unknownInLoop: walkOut.unknownInLoop
  };
}

function functionsUnder(root: GuessNode): GuessNode[] {
  if (root.type === 'function') return [root];
  return root.children.flatMap(functionsUnder);
}

export function analyze(root: GuessNode): Guess {
  const reports = functionsUnder(root).map(analyzeFunction);

  let best: string | null = null;
  let bestRank = -1;
  const reasons: string[] = [];
  let signals = 0;
  let unknownInLoop = false;

  for (const report of reports) {
    signals += report.signalCount;
    unknownInLoop = unknownInLoop || report.unknownInLoop;
    for (const [cls, reason] of report.classes) {
      reasons.push(`${report.name}: ${reason}`);
      const rank = CLASS_RANK.get(cls) ?? -1;
      if (rank > bestRank) {
        best = cls;
        bestRank = rank;
      }
    }
  }

  if (best === null) {
    return { guess_class: null, confidence: 'low', reasons: [] };
  }
  const confidence = signals === 1 && !unknownInLoop ? 'med' : 'low';
  return { guess_class: best, confidence, reasons };
}
