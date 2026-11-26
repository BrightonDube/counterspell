module.exports = {
  hop_limit: (n, links, ttl) => {
    if (n === 0) return 0;
    const adj = Array.from({ length: n }, () => []);
    for (const [a, b] of links) { if (0 <= a && a < n && 0 <= b && b < n) { adj[a].push(b); adj[b].push(a); } }
    const dist = new Map([[0, 0]]);
    const queue = [0];
    for (let i = 0; i < queue.length; i++) {
      const u = queue[i];
      if (dist.get(u) >= ttl) continue;
      for (const v of adj[u]) {
        if (!dist.has(v)) { dist.set(v, dist.get(u) + 1); queue.push(v); }
      }
    }
    return dist.size - 1;
  },
};
