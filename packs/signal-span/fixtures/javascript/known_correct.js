module.exports = {
  span_max: (edges, n) => {
    if (n <= 0) return 0;
    const adj = Array.from({ length: n }, () => []);
    for (const [a, b] of edges) {
      if (a >= 0 && a < n && b >= 0 && b < n) {
        adj[a].push(b);
        adj[b].push(a);
      }
    }
    const dist = new Array(n).fill(-1);
    dist[0] = 0;
    const queue = [0];
    for (let i = 0; i < queue.length; i += 1) {
      for (const y of adj[queue[i]]) {
        if (dist[y] < 0) {
          dist[y] = dist[queue[i]] + 1;
          queue.push(y);
        }
      }
    }
    return Math.max(...dist);
  },
};
