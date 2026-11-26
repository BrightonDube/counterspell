module.exports = {
  row_ways: (length) => {
    if (length <= 0) return 1;
    const d = new Array(length + 1).fill(0);
    d[0] = 1;
    for (let i = 1; i <= length; i += 1) {
      for (const p of [1, 2, 3]) {
        if (i - p >= 0) d[i] += d[i - p];
      }
    }
    return d[length];
  },
};
