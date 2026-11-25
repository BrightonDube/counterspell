module.exports = {
  smallest_largest: (offers, parts) => {
    const ok = (cap) => {
      let need = 1, cur = 0;
      for (const v of offers) {
        if (cur + v > cap) { need += 1; cur = 0; }
        cur += v;
      }
      return need <= parts;
    };
    let lo = Math.max(...offers), hi = offers.reduce((a, b) => a + b, 0);
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (ok(mid)) { hi = mid; } else { lo = mid + 1; }
    }
    return lo;
  },
};
