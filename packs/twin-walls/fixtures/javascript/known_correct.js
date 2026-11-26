module.exports = {
  twin_walls: (edges, n) => {
    const adj = Array.from({ length: Math.max(0, n) }, () => []);
    for (const [a, b] of edges) {
      if (a >= 0 && a < n && b >= 0 && b < n) {
        adj[a].push(b);
        adj[b].push(a);
      }
    }
    const colour = new Array(Math.max(0, n)).fill(null);
    for (let s = 0; s < n; s += 1) {
      if (colour[s] !== null) continue;
      colour[s] = 0;
      const stack = [s];
      while (stack.length) {
        const x = stack.pop();
        for (const y of adj[x]) {
          if (colour[y] === null) {
            colour[y] = 1 - colour[x];
            stack.push(y);
          } else if (colour[y] === colour[x]) {
            return 0;
          }
        }
      }
    }
    return 1;
  },
};
