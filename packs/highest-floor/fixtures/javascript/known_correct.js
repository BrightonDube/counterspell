module.exports = {
  highest_floor: (steps, reach) => {
    let lo = 0, hi = steps.length;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (steps[mid] <= reach) { lo = mid + 1; } else { hi = mid; }
    }
    return lo > 0 ? steps[lo - 1] : -1;
  },
};
