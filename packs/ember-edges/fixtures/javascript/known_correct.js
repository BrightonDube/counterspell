module.exports = {
  ember_edges: (embers, target) => {
    const lower = (mark) => {
      let lo = 0, hi = embers.length;
      while (lo < hi) {
        const mid = (lo + hi) >> 1;
        if (embers[mid] < mark) { lo = mid + 1; } else { hi = mid; }
      }
      return lo;
    };
    const first = lower(target);
    if (first === embers.length || embers[first] !== target) {
      return [-1, -1];
    }
    return [first, lower(target + 1) - 1];
  },
};
