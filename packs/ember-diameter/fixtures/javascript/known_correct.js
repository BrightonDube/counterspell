module.exports = {
  tree_diameter: (vals, left, right) => {
    const n = vals.length;
    if (n === 0) return 0;
    const heights = new Array(n).fill(-1), seen = new Set(), stack = [[0, false]];
    let best = 0;
    while (stack.length) {
      const [i, done] = stack.pop();
      if (seen.has(i)) continue;
      if (!done) {
        stack.push([i, true]);
        for (const arr of [left, right]) {
          const k = arr[i] >= 0 && arr[i] < n ? arr[i] : -1;
          if (k !== -1) stack.push([k, false]);
        }
        continue;
      }
      seen.add(i);
      const lo = left[i] >= 0 && left[i] < n ? left[i] : -1;
      const hi = right[i] >= 0 && right[i] < n ? right[i] : -1;
      const lh = lo !== -1 && heights[lo] >= 0 ? heights[lo] + 1 : 0;
      const rh = hi !== -1 && heights[hi] >= 0 ? heights[hi] + 1 : 0;
      if (lh + rh > best) best = lh + rh;
      heights[i] = Math.max(lh, rh);
    }
    return best;
  },
};
