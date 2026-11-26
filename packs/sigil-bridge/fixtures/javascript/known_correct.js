module.exports = {
  bridge_count: (edges, n) => {
    const adj = Array.from({ length: Math.max(0, n) }, () => []);
    let arcId = 0;
    for (const [a, b] of edges) {
      if (a >= 0 && a < n && b >= 0 && b < n && a !== b) {
        const idx = arcId;
        arcId += 1;
        adj[a].push([b, idx]);
        adj[b].push([a, idx]);
      }
    }
    const disc = new Array(Math.max(0, n)).fill(-1);
    const low = new Array(Math.max(0, n)).fill(0);
    let stamp = 0;
    let bridges = 0;
    const dfs = (u, entryArc) => {
      disc[u] = stamp;
      low[u] = stamp;
      stamp += 1;
      for (const [v, arc] of adj[u]) {
        if (arc === entryArc) continue;
        if (disc[v] === -1) {
          dfs(v, arc);
          low[u] = Math.min(low[u], low[v]);
          if (low[v] > disc[u]) bridges += 1;
        } else {
          low[u] = Math.min(low[u], disc[v]);
        }
      }
    };
    for (let s = 0; s < n; s += 1) {
      if (disc[s] === -1) dfs(s, -1);
    }
    return bridges;
  },
};
