module.exports = {
  find_in_rotated: (stores, target) => {
    let lo = 0, hi = stores.length - 1;
    while (lo <= hi) {
      const mid = (lo + hi) >> 1;
      if (stores[mid] === target) return mid;
      if (stores[lo] <= stores[mid]) {
        if (stores[lo] <= target && target < stores[mid]) { hi = mid - 1; } else { lo = mid + 1; }
      } else {
        if (stores[mid] < target && target <= stores[hi]) { lo = mid + 1; } else { hi = mid - 1; }
      }
    }
    return -1;
  },
};
