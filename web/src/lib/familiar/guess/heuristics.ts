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
  'zip'
]);

// Methods that make an in-loop step linear in the container (linear search and
// front-of-list mutation).
const LINEAR_METHODS = new Set([
  'contains',
  'find',
  'includes',
  'index',
  'indexof',
  'insert',
  'remove',
  'shift',
  'splice',
  'unshift'
]);

const SORT_CALLS = new Set(['sort', 'sorted', 'sort_by', 'sortby']);
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
  if (node.type === 'assign') out.push(node);
  for (const child of node.children) assignsIn(child, out);
  return out;
}

// A `//`, `/`, or `>>` by a constant whose other operand mentions one of the
// target names — `(lo + hi) // 2` halves the `lo`/`hi` range, `10 // 2` and
// `k // 2` shrink nobody's bound.
function hasHalvingOpOn(node: GuessNode, targets: Set<string>): boolean {
  if (
    node.type === 'binary_op' &&
    ['//', '/', '>>'].includes(node.text) &&
    node.children.some((operand) => operand.type === 'literal') &&
    node.children.some((operand) =>
      [...namesIn(operand, new Set())].some((name) => targets.has(name))
    )
  ) {
    return true;
  }
  return node.children.some((child) => hasHalvingOpOn(child, targets));
}

// True when a `while` body shrinks a variable the condition tests by a
// geometric factor — `x = x // 2`, `x >>= 1`, or the binary-search shape where
// a bound var takes `mid` and `mid` came from a halving, including updates
// nested inside `if`/`else`. An unrelated division (`half = n // 2; n = n - 1`)
// or a constant decrement (`n = n - (10 // 2)`) is not a halving loop.
function halvesBound(cond: GuessNode | undefined, body: GuessNode[]): boolean {
  if (!cond) return false;
  const condNames = namesIn(cond, new Set());
  const assigns = body.flatMap((node) => assignsIn(node));
  const halvingValues = new Set<string>();
  for (const node of assigns) {
    if (node.children.some((value) => hasHalvingOpOn(value, condNames))) {
      halvingValues.add(node.text);
    }
  }
  return assigns.some(
    (node) =>
      condNames.has(node.text) &&
      node.children.some(
        (value) =>
          hasHalvingOpOn(value, condNames) ||
          [...namesIn(value, new Set())].some((name) => halvingValues.has(name))
      )
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
