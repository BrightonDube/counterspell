module.exports = {
  bst_check: (vals, left, right) => {
    const n = vals.length;
    if (n === 0) return 1;
    const work = [[0, -Infinity, Infinity]], seen = new Set();
    while (work.length) {
      const [i, lo, hi] = work.pop();
      if (!(lo < vals[i] && vals[i] < hi)) return 0;
      const key = i + "," + lo + "," + hi;
      if (seen.has(key)) continue;
      seen.add(key);
      const l = left[i] >= 0 && left[i] < n ? left[i] : -1;
      const r = right[i] >= 0 && right[i] < n ? right[i] : -1;
      if (l !== -1) work.push([l, lo, vals[i]]);
      if (r !== -1) work.push([r, vals[i], hi]);
    }
    return 1;
  },
};
