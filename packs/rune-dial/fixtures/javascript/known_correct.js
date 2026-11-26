module.exports = {
  run_median: (marks) => {
    const out = [];
    const seen = [];
    for (const v of marks) {
      let i = 0;
      while (i < seen.length && seen[i] < v) i += 1;
      seen.splice(i, 0, v);
      const n = seen.length;
      out.push(n % 2 ? 2 * seen[(n - 1) / 2] : seen[n / 2 - 1] + seen[n / 2]);
    }
    return out;
  },
};
