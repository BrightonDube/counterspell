module.exports = {
  split_rite: (n, links, down) => {
    const dead = new Set(down);
    const adj = Array.from({ length: n }, () => []);
    links.forEach(([a, b], i) => {
      if (!dead.has(i) && 0 <= a && a < n && 0 <= b && b < n) { adj[a].push(b); adj[b].push(a); }
    });
    const seen = new Set();
    let quorum = 0;
    for (let s = 0; s < n; s++) {
      if (seen.has(s)) continue;
      let size = 0;
      const queue = [s];
      seen.add(s);
      for (let i = 0; i < queue.length; i++) {
        size++;
        for (const v of adj[queue[i]]) {
          if (!seen.has(v)) { seen.add(v); queue.push(v); }
        }
      }
      if (size * 2 > n) quorum++;
    }
    return quorum;
  },
};
