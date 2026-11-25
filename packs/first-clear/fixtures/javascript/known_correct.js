module.exports = {
  first_clear: (heights, mark) => {
    let lo = 0, hi = heights.length;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (heights[mid] < mark) { lo = mid + 1; } else { hi = mid; }
    }
    return lo;
  },
};
