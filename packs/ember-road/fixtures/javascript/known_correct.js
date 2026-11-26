module.exports = {
  road_toll: (edges, weights, n) => {
    if (n <= 0) return -1;
    const adj = Array.from({ length: n }, () => []);
    for (let k = 0; k < Math.min(edges.length, weights.length); k += 1) {
      const [a, b] = edges[k];
      const c = weights[k];
      if (a >= 0 && a < n && b >= 0 && b < n) {
        adj[a].push([b, c]);
        adj[b].push([a, c]);
      }
    }
    const dist = new Array(n).fill(Infinity);
    dist[0] = 0;
    const pq = [[0, 0]];
    while (pq.length) {
      pq.sort((x, y) => x[0] - y[0]);
      const [du, u] = pq.shift();
      if (du > dist[u]) continue;
      for (const [v, c] of adj[u]) {
        const nd = du + c;
        if (nd < dist[v]) {
          dist[v] = nd;
          pq.push([nd, v]);
        }
      }
    }
    return dist[n - 1] === Infinity ? -1 : dist[n - 1];
  },
};
