module.exports = {
  split_count: (marks) => {
    const total = marks.reduce((a, b) => a + b, 0);
    if (total % 2) return 0;
    const half = total / 2;
    const go = (i, need) => {
      if (need === 0) return 1;
      if (i === marks.length) return 0;
      return go(i + 1, need - marks[i]) + go(i + 1, need);
    };
    if (half === 0) {
      return Math.max(0, 2 ** marks.length - 2) / 2;
    }
    return Math.floor(go(0, half) / 2);
  },
};
