module.exports = {
  ember_path: (vals, left, right, target) => {
    const n = vals.length;
    if (n === 0) return 0;
    const work = [[0, vals[0]]], seen = new Set();
    while (work.length) {
      const [i, total] = work.pop();
      const key = i + "," + total;
      if (seen.has(key)) continue;
      seen.add(key);
      const lo = left[i] >= 0 && left[i] < n ? left[i] : -1;
      const hi = right[i] >= 0 && right[i] < n ? right[i] : -1;
      if (lo === -1 && hi === -1) {
        if (total === target) return 1;
        continue;
      }
      if (lo !== -1) work.push([lo, total + vals[lo]]);
      if (hi !== -1) work.push([hi, total + vals[hi]]);
    }
    return 0;
  },
};
