module.exports = {
  isle_count: (edges, n) => {
    const adj = Array.from({ length: Math.max(0, n) }, () => []);
    for (const [a, b] of edges) {
      if (a >= 0 && a < n && b >= 0 && b < n) {
        adj[a].push(b);
        adj[b].push(a);
      }
    }
    const seen = new Array(Math.max(0, n)).fill(false);
    let count = 0;
    for (let s = 0; s < n; s += 1) {
      if (seen[s]) continue;
      count += 1;
      seen[s] = true;
      const stack = [s];
      while (stack.length) {
        const x = stack.pop();
        for (const y of adj[x]) {
          if (!seen[y]) {
            seen[y] = true;
            stack.push(y);
          }
        }
      }
    }
    return count;
  },
};
