module.exports = {
  widest_ring: (vals, left, right) => {
    const n = vals.length;
    if (n === 0) return 0;
    const counts = new Map(), seen = new Set(), queue = [[0, 0]];
    while (queue.length) {
      const [i, d] = queue.shift();
      if (seen.has(i)) continue;
      seen.add(i);
      counts.set(d, (counts.get(d) || 0) + 1);
      const lo = left[i] >= 0 && left[i] < n ? left[i] : -1;
      const hi = right[i] >= 0 && right[i] < n ? right[i] : -1;
      if (lo !== -1) queue.push([lo, d + 1]);
      if (hi !== -1) queue.push([hi, d + 1]);
    }
    return Math.max(...counts.values());
  },
};
