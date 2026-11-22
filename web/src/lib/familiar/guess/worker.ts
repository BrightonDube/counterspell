/// <reference lib="webworker" />
// Live-typing guess worker (efficiency-familiar T48, R14.1/R14.6).
//
// Loads web-tree-sitter and the Python and JavaScript grammars on the first
// message — nothing is fetched at page load — then runs the pure mapper and
// heuristics. Everything stays local: the only requests are the bundled .wasm
// assets, and nothing leaves the browser.

import { Language, Parser } from 'web-tree-sitter';
import runtimeWasm from 'web-tree-sitter/web-tree-sitter.wasm?url';
import pythonWasm from 'tree-sitter-python/tree-sitter-python.wasm?url';
import javascriptWasm from 'tree-sitter-javascript/tree-sitter-javascript.wasm?url';

import { mapNode } from './cst';
import { analyze, type Guess } from './heuristics';

interface GuessRequest {
  seq: number;
  code: string;
  language: string;
}

interface GuessResponse {
  seq: number;
  guess: Guess | null;
}

declare const self: DedicatedWorkerGlobalScope;

const GRAMMARS: Record<string, string> = {
  python: pythonWasm,
  javascript: javascriptWasm,
  typescript: javascriptWasm
};

let runtime: Promise<void> | null = null;
let parser: Parser | null = null;
let grammarLanguage = '';

async function ensureParser(language: string): Promise<Parser | null> {
  const wasmUrl = GRAMMARS[language];
  if (!wasmUrl) return null;
  runtime ??= Parser.init({ locateFile: () => runtimeWasm });
  await runtime;
  parser ??= new Parser();
  if (grammarLanguage !== language) {
    const grammar = await Language.load(wasmUrl);
    parser.setLanguage(grammar);
    grammarLanguage = language;
  }
  return parser;
}

self.onmessage = (event: MessageEvent<GuessRequest>) => {
  const { seq, code, language } = event.data;
  const reply = (guess: Guess | null) => {
    const response: GuessResponse = { seq, guess };
    self.postMessage(response);
  };
  void (async () => {
    try {
      const instance = await ensureParser(language);
      if (!instance) {
        reply(null);
        return;
      }
      const tree = instance.parse(code);
      const guess = tree ? analyze(mapNode(tree.rootNode)) : null;
      tree?.delete();
      reply(guess);
    } catch {
      reply(null);
    }
  })();
};
