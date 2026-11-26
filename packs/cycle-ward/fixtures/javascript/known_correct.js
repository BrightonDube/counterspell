module.exports = {
  cycle_check: (edges, n) => {
    const parent = Array.from({ length: Math.max(0, n) }, (_, i) => i);
    const find = (x) => {
      while (parent[x] !== x) {
        parent[x] = parent[parent[x]];
        x = parent[x];
      }
      return x;
    };
    for (const [a, b] of edges) {
      if (!(a >= 0 && a < n && b >= 0 && b < n)) continue;
      if (a === b) return 1;
      const ra = find(a);
      const rb = find(b);
      if (ra === rb) return 1;
      parent[ra] = rb;
    }
    return 0;
  },
};
