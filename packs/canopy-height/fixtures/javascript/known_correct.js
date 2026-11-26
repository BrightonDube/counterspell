module.exports = {
  tree_height: (vals, left, right) => {
    const n = vals.length;
    if (n === 0) return 0;
    let best = 0;
    const seen = new Set(), stack = [[0, 1]];
    while (stack.length) {
      const [i, d] = stack.pop();
      if (seen.has(i)) continue;
      seen.add(i);
      if (d > best) best = d;
      const lo = left[i] >= 0 && left[i] < n ? left[i] : -1;
      const hi = right[i] >= 0 && right[i] < n ? right[i] : -1;
      if (lo !== -1) stack.push([lo, d + 1]);
      if (hi !== -1) stack.push([hi, d + 1]);
    }
    return best;
  },
};
