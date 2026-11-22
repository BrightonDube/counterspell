// CST -> GuessNode mapping for the live-typing guess (T48, R14.2).
//
// Pure mapping over web-tree-sitter's SyntaxNode for the Python and
// JavaScript grammars. Kept separate from worker.ts so vitest can parse real
// source in Node; worker.ts only owns WASM loading and the message loop.

import type { Node as SyntaxNode } from 'web-tree-sitter';

import type { GuessNode } from './heuristics';

function anonymousText(node: SyntaxNode): string {
  return node.children
    .filter((child) => !child.isNamed)
    .map((child) => child.type)
    .join(' ');
}

function decoratorName(node: SyntaxNode): string {
  // `functools.lru_cache` reduces to `lru_cache`; calls unwrap to the callee.
  let inner = node;
  while (inner.namedChildren.length > 0) {
    const next =
      inner.childForFieldName('function') ??
      inner.childForFieldName('attribute') ??
      inner.namedChildren[inner.namedChildren.length - 1];
    if (next === inner) break;
    inner = next;
  }
  return inner.type === 'identifier'
    ? inner.text
    : (inner.text.split('.').pop() ?? inner.text);
}

function paramNames(node: SyntaxNode, out: GuessNode[]): GuessNode[] {
  if (
    node.type === 'identifier' ||
    node.type === 'parameter_identifier' ||
    node.type === 'property_identifier'
  ) {
    out.push({ type: 'name', text: node.text, children: [] });
    return out;
  }
  for (const child of node.namedChildren) paramNames(child, out);
  return out;
}

function functionNode(node: SyntaxNode, name: string): GuessNode {
  const params = node.childForFieldName('parameters');
  const nameNode = node.childForFieldName('name');
  const children: GuessNode[] = [
    { type: 'params', text: '', children: params ? paramNames(params, []) : [] }
  ];
  for (const child of node.namedChildren) {
    if (child === params || child === nameNode) continue;
    children.push(...mapChildren(child));
  }
  return { type: 'function', text: name, children };
}

function callNode(node: SyntaxNode): GuessNode {
  const callee = node.childForFieldName('function');
  if (!callee) return flat(node);
  const args = node.childForFieldName('arguments');
  const argNodes = args ? mapChildren(args) : [];
  if (callee.type === 'identifier') {
    return { type: 'call', text: callee.text, children: argNodes };
  }
  if (callee.type === 'attribute' || callee.type === 'member_expression') {
    const recv = callee.childForFieldName('object');
    const attr =
      callee.childForFieldName('attribute') ??
      callee.childForFieldName('property');
    return {
      type: 'member_call',
      text: attr?.text ?? callee.text,
      children: [...(recv ? [mapExpr(recv)] : []), ...argNodes]
    };
  }
  return { type: 'call', text: callee.text, children: argNodes };
}

function mapChildren(node: SyntaxNode): GuessNode[] {
  return node.namedChildren.flatMap(mapChild);
}

function mapChild(child: SyntaxNode): GuessNode[] {
  const mapped = mapNode(child);
  return mapped.type === 'skip' ? mapChildren(child) : [mapped];
}

const LITERAL_TYPES = new Set([
  'integer',
  'float',
  'string',
  'concatenated_string',
  'number',
  'template_string',
  'true',
  'false',
  'none',
  'null',
  'undefined',
  'list',
  'list_pattern',
  'dictionary',
  'set',
  'tuple',
  'object',
  'array',
  'regex'
]);

const SKIP_TYPES = new Set([
  'comment',
  'expression_statement',
  'block',
  'statement_block',
  'else_clause',
  'elif_clause',
  'argument_list',
  'arguments',
  'parenthesized_expression'
]);

function leftName(left: SyntaxNode | null): string {
  if (!left) return '';
  if (left.type === 'identifier') return left.text;
  if (left.type === 'attribute' || left.type === 'member_expression') {
    return left.childForFieldName('object')?.text ?? left.text;
  }
  if (left.type === 'pattern_list' || left.type === 'tuple_pattern') {
    return left.namedChildren[0]?.text ?? left.text;
  }
  return left.text;
}

// Skipped wrappers (parentheses, expression statements) still need their one
// child visible as a single node where a slot takes exactly one expression.
function mapExpr(node: SyntaxNode): GuessNode {
  const mapped = mapNode(node);
  if (mapped.type !== 'skip') return mapped;
  return { type: 'expr', text: '', children: mapChildren(node) };
}

function flat(node: SyntaxNode): GuessNode {
  return { type: node.type, text: '', children: mapChildren(node) };
}

export function mapNode(node: SyntaxNode): GuessNode {
  // `flatMap(mapChild)`, not `map(mapNode)`: a wrapped child such as a
  // parenthesized expression must flatten, not become an empty skip node.
  const kids = () => node.namedChildren.flatMap(mapChild);
  switch (node.type) {
    case 'module':
    case 'program':
      return { type: 'module', text: '', children: kids() };

    case 'function_definition':
    case 'function_declaration':
    case 'generator_function_declaration':
    case 'method_definition':
      return functionNode(node, node.childForFieldName('name')?.text ?? '');

    case 'decorated_definition': {
      // Decorators sit beside the function in the CST; fold them onto it so
      // the GuessNode shape matches the tests' `fn(...)` builder.
      const inner = node.namedChildren.find(
        (child) => child.type === 'function_definition'
      );
      if (!inner) return flat(node);
      const decorators = node.namedChildren
        .filter((child) => child.type === 'decorator')
        .map((d) => ({
          type: 'decorator',
          text: decoratorName(d),
          children: []
        }));
      const fn = functionNode(
        inner,
        inner.childForFieldName('name')?.text ?? ''
      );
      return {
        ...fn,
        children: [fn.children[0], ...decorators, ...fn.children.slice(1)]
      };
    }

    case 'arrow_function':
    case 'function':
    case 'function_expression':
    case 'lambda':
      return functionNode(node, '');

    case 'for_statement': {
      // Python's `right` and JS `for..in/of`'s `right` are the iterable. A
      // C-style `for` has init/condition/increment instead; they together act
      // as the input-dependent signal.
      const iterable =
        node.childForFieldName('right') ?? node.childForFieldName('condition');
      const extra = [
        node.childForFieldName('initializer'),
        node.childForFieldName('increment')
      ].filter((c): c is SyntaxNode => c !== null && c !== iterable);
      const body = node.childForFieldName('body');
      return {
        type: 'for',
        text: '',
        children: [
          ...(iterable || extra.length
            ? [
                {
                  type: 'for_head',
                  text: '',
                  children: [
                    ...(iterable ? [mapExpr(iterable)] : []),
                    ...extra.map(mapExpr)
                  ]
                }
              ]
            : []),
          ...(body ? mapChild(body) : [])
        ]
      };
    }
    case 'for_in_statement': {
      const iterable = node.childForFieldName('right');
      const body = node.childForFieldName('body');
      return {
        type: 'for',
        text: '',
        children: [
          ...(iterable ? [mapExpr(iterable)] : []),
          ...(body ? mapChild(body) : [])
        ]
      };
    }

    case 'while_statement':
    case 'do_statement': {
      const cond = node.childForFieldName('condition');
      const body = node.childForFieldName('body');
      return {
        type: 'while',
        text: '',
        children: [
          ...(cond ? [mapExpr(cond)] : []),
          ...(body ? mapChild(body) : [])
        ]
      };
    }

    case 'if_statement': {
      const cond = node.childForFieldName('condition');
      const consequence = node.childForFieldName('consequence');
      const alternative = node.childForFieldName('alternative');
      return {
        type: 'if',
        text: '',
        children: [
          ...(cond ? [mapExpr(cond)] : []),
          ...(consequence ? mapChild(consequence) : []),
          ...(alternative ? mapChild(alternative) : [])
        ]
      };
    }

    case 'assignment':
    case 'assignment_expression': {
      const left = node.childForFieldName('left');
      const right = node.childForFieldName('right');
      return {
        type: 'assign',
        text: leftName(left),
        children: right ? [mapExpr(right)] : []
      };
    }

    case 'augmented_assignment':
    case 'augmented_assignment_expression': {
      const left = node.childForFieldName('left');
      const right = node.childForFieldName('right');
      const name = leftName(left);
      const operator = anonymousText(node).replace('=', '');
      return {
        type: 'assign',
        text: name,
        children: [
          {
            type: 'binary_op',
            text: operator,
            children: [
              { type: 'name', text: name, children: [] },
              ...(right ? [mapExpr(right)] : [])
            ]
          }
        ]
      };
    }

    case 'variable_declarator':
    case 'lexical_declaration':
    case 'variable_declaration': {
      // `const f = () => ...` binds the function's name for self-call counts.
      const name = node.childForFieldName('name');
      const value = node.childForFieldName('value');
      if (
        name &&
        value &&
        (value.type === 'arrow_function' ||
          value.type === 'function' ||
          value.type === 'function_expression')
      ) {
        return {
          type: 'assign',
          text: name.text,
          children: [functionNode(value, name.text)]
        };
      }
      if (name && value) {
        return {
          type: 'assign',
          text: name.text,
          children: [mapExpr(value)]
        };
      }
      return flat(node);
    }

    case 'update_expression': {
      const operand = node.namedChildren[0];
      const name = operand?.text ?? '';
      return {
        type: 'assign',
        text: name,
        children: [
          {
            type: 'binary_op',
            text: anonymousText(node) === '--' ? '-' : '+',
            children: [
              { type: 'name', text: name, children: [] },
              { type: 'literal', text: '1', children: [] }
            ]
          }
        ]
      };
    }

    case 'binary_operator':
    case 'binary_expression':
    case 'boolean_operator':
      return {
        type: 'binary_op',
        text: anonymousText(node),
        children: kids()
      };

    case 'comparison_operator':
      return {
        type: 'compare',
        text: anonymousText(node),
        children: kids()
      };

    case 'call':
    case 'call_expression':
      return callNode(node);

    case 'new_expression': {
      const ctor = node.childForFieldName('constructor');
      const args = node.childForFieldName('arguments');
      return {
        type: 'call',
        text: ctor?.text ?? '',
        children: args ? mapChildren(args) : []
      };
    }

    case 'attribute':
    case 'member_expression': {
      const object = node.childForFieldName('object');
      return { type: 'member', text: object?.text ?? node.text, children: [] };
    }

    case 'identifier':
    case 'property_identifier':
    case 'shorthand_property_identifier':
      return { type: 'name', text: node.text, children: [] };

    case 'return_statement':
      return { type: 'return', text: '', children: kids() };

    case 'subscript':
    case 'subscript_expression':
    case 'unary_operator':
    case 'unary_expression':
    case 'not_operator':
    case 'conditional_expression':
    case 'ternary_expression':
      return flat(node);

    default:
      if (LITERAL_TYPES.has(node.type)) {
        return { type: 'literal', text: node.text, children: [] };
      }
      if (SKIP_TYPES.has(node.type)) {
        return { type: 'skip', text: '', children: [] };
      }
      return flat(node);
  }
}
