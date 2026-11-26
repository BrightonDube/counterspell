module.exports = {
  mirror_check: (vals, left, right) => {
    const n = vals.length;
    if (n === 0) return 1;
    const kid = (arr, i) => (i < arr.length && arr[i] >= 0 && arr[i] < n ? arr[i] : -1);
    const work = [[kid(left, 0), kid(right, 0)]], seen = new Set();
    while (work.length) {
      const [a, b] = work.pop();
      const key = a + "," + b;
      if (seen.has(key)) continue;
      seen.add(key);
      if (a === -1 && b === -1) continue;
      if (a === -1 || b === -1 || vals[a] !== vals[b]) return 0;
      work.push([kid(left, a), kid(right, b)]);
      work.push([kid(right, a), kid(left, b)]);
    }
    return 1;
  },
};
