module.exports = {
  count_leaves: (vals, left, right) => {
    const n = vals.length, seen = new Set(), stack = n ? [0] : [];
    let leaves = 0;
    while (stack.length) {
      const i = stack.pop();
      if (seen.has(i)) continue;
      seen.add(i);
      const lo = left[i] >= 0 && left[i] < n ? left[i] : -1;
      const hi = right[i] >= 0 && right[i] < n ? right[i] : -1;
      if (lo === -1 && hi === -1) { leaves += 1; continue; }
      if (lo !== -1) stack.push(lo);
      if (hi !== -1) stack.push(hi);
    }
    return leaves;
  },
};
