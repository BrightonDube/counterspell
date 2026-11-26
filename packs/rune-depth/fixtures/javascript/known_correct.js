module.exports = {
  rune_depth: (edges, n) => {
    const adj = Array.from({ length: Math.max(0, n) }, () => []);
    let selfOrder = false;
    for (const [a, b] of edges) {
      if (!(a >= 0 && a < n && b >= 0 && b < n)) continue;
      if (a === b) {
        selfOrder = true;
        continue;
      }
      if (!adj[a].includes(b)) {
        adj[a].push(b);
      }
    }
    if (selfOrder) return 0;
    const state = new Array(Math.max(0, n)).fill(0);
    const memo = new Array(Math.max(0, n)).fill(0);
    const dfs = (u) => {
      state[u] = 1;
      let best = 0;
      for (const w of adj[u]) {
        if (state[w] === 1) throw new Error('cycle');
        if (state[w] === 0) dfs(w);
        best = Math.max(best, memo[w]);
      }
      state[u] = 2;
      memo[u] = best + 1;
    };
    try {
      for (let v = 0; v < n; v += 1) {
        if (state[v] === 0) dfs(v);
      }
    } catch (e) {
      return 0;
    }
    return n > 0 ? Math.max(...memo) : 0;
  },
};
