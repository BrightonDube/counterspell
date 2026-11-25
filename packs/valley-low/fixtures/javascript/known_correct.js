module.exports = {
  lowest_mark: (rotated) => {
    let lo = 0, hi = rotated.length - 1;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (rotated[mid] > rotated[hi]) { lo = mid + 1; } else { hi = mid; }
    }
    return rotated[lo];
  },
};
