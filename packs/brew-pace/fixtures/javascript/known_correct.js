module.exports = {
  min_pace: (brews, hours) => {
    let lo = 1, hi = Math.max(...brews);
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      let need = 0;
      for (const t of brews) need += Math.ceil(t / mid);
      if (need <= hours) { hi = mid; } else { lo = mid + 1; }
    }
    return lo;
  },
};
