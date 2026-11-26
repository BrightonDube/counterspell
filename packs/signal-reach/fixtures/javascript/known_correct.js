module.exports = {
  reach_count: (edges, n) => {
    if (n <= 0) return 0;
    const adj = Array.from({ length: n }, () => []);
    for (const [a, b] of edges) {
      if (a >= 0 && a < n && b >= 0 && b < n) {
        adj[a].push(b);
        adj[b].push(a);
      }
    }
    const seen = new Array(n).fill(false);
    seen[0] = true;
    const stack = [0];
    let count = 0;
    while (stack.length) {
      const x = stack.pop();
      count += 1;
      for (const y of adj[x]) {
        if (!seen[y]) {
          seen[y] = true;
          stack.push(y);
        }
      }
    }
    return count;
  },
};
