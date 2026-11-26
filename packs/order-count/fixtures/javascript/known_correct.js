module.exports = {
  order_count: (edges, n) => {
    const adj = Array.from({ length: Math.max(0, n) }, () => []);
    const indeg = new Array(Math.max(0, n)).fill(0);
    let selfOrder = false;
    for (const [a, b] of edges) {
      if (!(a >= 0 && a < n && b >= 0 && b < n)) continue;
      if (a === b) {
        selfOrder = true;
        continue;
      }
      if (!adj[a].includes(b)) {
        adj[a].push(b);
        indeg[b] += 1;
      }
    }
    if (selfOrder) return 0;
    let total = 0;
    const go = (k) => {
      if (k === n) {
        total += 1;
        return;
      }
      for (let v = 0; v < n; v += 1) {
        if (indeg[v] === 0) {
          indeg[v] = -1;
          for (const w of adj[v]) indeg[w] -= 1;
          go(k + 1);
          for (const w of adj[v]) indeg[w] += 1;
          indeg[v] = 0;
        }
      }
    };
    go(0);
    return total;
  },
};
