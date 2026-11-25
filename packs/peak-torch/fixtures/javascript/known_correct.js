module.exports = {
  peak_torch: (heights) => {
    let lo = 0, hi = heights.length - 1;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (heights[mid] < heights[mid + 1]) { lo = mid + 1; } else { hi = mid; }
    }
    return lo;
  },
};
