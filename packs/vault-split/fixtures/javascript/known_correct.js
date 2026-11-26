module.exports = {
  split_diff: (marks) => {
    const total = marks.reduce((a, b) => a + b, 0);
    const half = Math.floor(total / 2);
    const d = new Array(half + 1).fill(false);
    d[0] = true;
    for (const m of marks) {
      for (let i = half; i >= m; i -= 1) {
        if (d[i - m]) d[i] = true;
      }
    }
    let best = 0;
    for (let i = half; i >= 0; i -= 1) {
      if (d[i]) { best = i; break; }
    }
    return total - 2 * best;
  },
};
