// Temporary browser harness for ide-workspace browser checks (not shipped).
// Serves /api/problems/{slug} and /api/languages from in-memory fixtures and
// proxies everything else to the vite dev server on :5173. The workspace page
// then renders its real shell end to end.
import http from 'node:http';

const VITE = 'http://127.0.0.1:5173';
const PORT = 5199;

const problem = (slug, title) => ({
  slug,
  title,
  track: { slug: 'arrays', name: 'Crypt of Arrays' },
  concepts: [{ slug: 'loops', name: 'Loops' }],
  difficulty: 'intro',
  statement_markdown:
    'Given an array of integers, return the indices of the two numbers that add to the target.',
  constraints: ['2 <= nums.length <= 10^4', 'One valid answer exists.'],
  signature: {
    function_name: 'two_sum',
    parameters: [
      { name: 'nums', type: 'list<int>' },
      { name: 'target', type: 'int' }
    ],
    return_type: 'list<int>'
  },
  samples: [{ input: '[[2, 7], 9]', expected: '[0, 1]' }],
  starters: { python: 'def two_sum(nums, target):\n    pass\n' },
  enabled_languages: ['python', 'javascript']
});

const problems = {
  'two-sum': problem('two-sum', 'Two Sum'),
  'count-pairs': problem('count-pairs', 'Count Pairs')
};

const languages = [
  {
    id: 'python',
    display_name: 'Python',
    enabled: true,
    file_name: 'solution.py',
    max_source_bytes: 65536,
    supports_run: true,
    supports_submit: true
  },
  {
    id: 'javascript',
    display_name: 'JavaScript',
    enabled: true,
    file_name: 'solution.js',
    max_source_bytes: 65536,
    supports_run: true,
    supports_submit: true
  }
];

const server = http.createServer((req, res) => {
  const url = new URL(req.url ?? '/', `http://localhost:${PORT}`);
  const match = url.pathname.match(/^\/api\/problems\/([a-z0-9-]+)$/);
  if (match && problems[match[1]]) {
    res.writeHead(200, { 'content-type': 'application/json' });
    res.end(JSON.stringify(problems[match[1]]));
    return;
  }
  if (url.pathname === '/api/languages') {
    res.writeHead(200, { 'content-type': 'application/json' });
    res.end(JSON.stringify(languages));
    return;
  }
  const upstream = http.request(
    VITE + (req.url ?? '/'),
    { method: req.method, headers: req.headers },
    (proxy) => {
      res.writeHead(proxy.statusCode ?? 502, proxy.headers);
      proxy.pipe(res);
    }
  );
  upstream.on('error', () => {
    res.writeHead(502);
    res.end('vite dev is not reachable');
  });
  req.pipe(upstream);
});

server.listen(PORT, '127.0.0.1', () => {
  console.log(`e2e stub on http://127.0.0.1:${PORT}`);
});
