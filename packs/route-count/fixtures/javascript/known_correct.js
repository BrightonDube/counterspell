module.exports = {
  route_count: (n, links) => {
    if (n === 0) return 0;
    const adj = Array.from({ length: n }, () => []);
    for (const [a, b] of links) { if (0 <= a && a < n && 0 <= b && b < n) { adj[a].push(b); adj[b].push(a); } }
    const seen = new Set([0]);
    const queue = [0];
    for (let i = 0; i < queue.length; i++) {
      for (const v of adj[queue[i]]) {
        if (!seen.has(v)) { seen.add(v); queue.push(v); }
      }
    }
    return seen.size - 1;
  },
};
