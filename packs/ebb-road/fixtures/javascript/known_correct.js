const RING = -10 ** 15;
module.exports = {
  ebb_toll: (edges, weights, n) => {
    if (n <= 0) return -1;
    const arcs = [];
    for (let k = 0; k < Math.min(edges.length, weights.length); k += 1) {
      const [a, b] = edges[k];
      const c = weights[k];
      if (a >= 0 && a < n && b >= 0 && b < n) {
        arcs.push([a, b, c]);
      }
    }
    const dist = new Array(n).fill(Infinity);
    dist[0] = 0;
    for (let i = 0; i < n - 1; i += 1) {
      for (const [a, b, c] of arcs) {
        if (dist[a] < Infinity && dist[a] + c < dist[b]) {
          dist[b] = dist[a] + c;
        }
      }
    }
    const marked = new Array(n).fill(false);
    for (let i = 0; i < n; i += 1) {
      for (const [a, b, c] of arcs) {
        if ((dist[a] < Infinity && dist[a] + c < dist[b]) || marked[a]) {
          marked[b] = true;
        }
      }
    }
    if (marked[n - 1]) return RING;
    return dist[n - 1] === Infinity ? -1 : dist[n - 1];
  },
};
