module.exports = {
  hop_toll: (edges, weights, n, k) => {
    if (n <= 0 || k < 0) return -1;
    const adj = Array.from({ length: n }, () => []);
    for (let i = 0; i < Math.min(edges.length, weights.length); i += 1) {
      const [a, b] = edges[i];
      const c = weights[i];
      if (a >= 0 && a < n && b >= 0 && b < n) {
        adj[a].push([b, c]);
        adj[b].push([a, c]);
      }
    }
    const dist = Array.from({ length: n }, () => new Array(k + 1).fill(Infinity));
    dist[0][0] = 0;
    const pq = [[0, 0, 0]];
    while (pq.length) {
      pq.sort((x, y) => x[0] - y[0]);
      const [du, u, h] = pq.shift();
      if (du > dist[u][h]) continue;
      for (const [v, c] of adj[u]) {
        if (h + 1 <= k && du + c < dist[v][h + 1]) {
          dist[v][h + 1] = du + c;
          pq.push([dist[v][h + 1], v, h + 1]);
        }
      }
    }
    const best = Math.min(...dist[n - 1]);
    return best === Infinity ? -1 : best;
  },
};
