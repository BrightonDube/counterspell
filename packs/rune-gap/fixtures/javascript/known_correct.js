module.exports = {
  gap_count: (marks, cap) => {
    const go = (i, left) => {
      if (left < 0) return 0;
      if (i === marks.length) return 1;
      return go(i + 1, left - marks[i]) + go(i + 1, left);
    };
    return go(0, cap);
  },
};
