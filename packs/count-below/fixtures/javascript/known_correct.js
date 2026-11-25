module.exports = {
  count_below: (marks, toll) => {
    let lo = 0, hi = marks.length;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (marks[mid] < toll) { lo = mid + 1; } else { hi = mid; }
    }
    return lo;
  },
};
