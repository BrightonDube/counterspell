module.exports = {
  find_ember: (embers, target) => {
    let lo = 0, hi = embers.length;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (embers[mid] < target) { lo = mid + 1; } else { hi = mid; }
    }
    return lo < embers.length && embers[lo] === target ? lo : -1;
  },
};
