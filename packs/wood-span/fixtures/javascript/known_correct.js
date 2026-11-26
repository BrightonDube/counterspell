module.exports = {
  span_toll: (edges, weights, n) => {
    if (n <= 0) return 0;
    const parent = Array.from({ length: n }, (_, i) => i);
    const find = (x) => {
      while (parent[x] !== x) {
        parent[x] = parent[parent[x]];
        x = parent[x];
      }
      return x;
    };
    const arcs = [];
    for (let k = 0; k < Math.min(edges.length, weights.length); k += 1) {
      const [a, b] = edges[k];
      const c = weights[k];
      if (a >= 0 && a < n && b >= 0 && b < n && a !== b) {
        arcs.push([c, a, b]);
      }
    }
    arcs.sort((x, y) => x[0] - y[0]);
    let total = 0;
    let used = 0;
    for (const [c, a, b] of arcs) {
      const ra = find(a);
      const rb = find(b);
      if (ra !== rb) {
        parent[ra] = rb;
        total += c;
        used += 1;
      }
    }
    return used === n - 1 ? total : -1;
  },
};
