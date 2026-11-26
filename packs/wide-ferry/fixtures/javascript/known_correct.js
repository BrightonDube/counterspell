module.exports = {
  ferry_load: (edges, weights, n) => {
    if (n <= 1) return n === 1 ? 0 : -1;
    const adj = Array.from({ length: n }, () => []);
    for (let k = 0; k < Math.min(edges.length, weights.length); k += 1) {
      const [a, b] = edges[k];
      const c = weights[k];
      if (a >= 0 && a < n && b >= 0 && b < n) {
        adj[a].push([b, c]);
        adj[b].push([a, c]);
      }
    }
    const best = new Array(n).fill(-1);
    best[0] = Number.MAX_SAFE_INTEGER;
    const pq = [[best[0], 0]];
    while (pq.length) {
      pq.sort((x, y) => y[0] - x[0]);
      const [cap, u] = pq.shift();
      if (cap < best[u]) continue;
      for (const [v, c] of adj[u]) {
        const cand = Math.min(cap, c);
        if (cand > best[v]) {
          best[v] = cand;
          pq.push([cand, v]);
        }
      }
    }
    return best[n - 1];
  },
};
