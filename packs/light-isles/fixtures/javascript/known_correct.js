module.exports = {
  light_isles: (edges, weights, n, t) => {
    const parent = Array.from({ length: Math.max(0, n) }, (_, i) => i);
    const find = (x) => {
      while (parent[x] !== x) {
        parent[x] = parent[parent[x]];
        x = parent[x];
      }
      return x;
    };
    for (let k = 0; k < Math.min(edges.length, weights.length); k += 1) {
      const [a, b] = edges[k];
      const c = weights[k];
      if (c <= t && a >= 0 && a < n && b >= 0 && b < n) {
        const ra = find(a);
        const rb = find(b);
        if (ra !== rb) parent[ra] = rb;
      }
    }
    const roots = new Set();
    for (let i = 0; i < n; i += 1) roots.add(find(i));
    return roots.size;
  },
};
