module.exports = {
  tree_sum: (vals, left, right) => {
    const n = vals.length, seen = new Set(), stack = n ? [0] : [];
    let total = 0;
    while (stack.length) {
      const i = stack.pop();
      if (seen.has(i)) continue;
      seen.add(i);
      total += vals[i];
      const lo = left[i] >= 0 && left[i] < n ? left[i] : -1;
      const hi = right[i] >= 0 && right[i] < n ? right[i] : -1;
      if (lo !== -1) stack.push(lo);
      if (hi !== -1) stack.push(hi);
    }
    return total;
  },
};
