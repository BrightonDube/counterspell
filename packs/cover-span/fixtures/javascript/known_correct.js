module.exports = {
  span_cover: (spans, t) => {
    const s = [...spans].sort((x, y) => x[0] - y[0] || x[1] - y[1]);
    let front = 0;
    let i = 0;
    let n = 0;
    while (front <= t) {
      let best = -1;
      while (i < s.length && s[i][0] <= front) {
        best = Math.max(best, s[i][1]);
        i += 1;
      }
      if (best < front) return -1;
      n += 1;
      if (best >= t) return n;
      front = best + 1;
    }
    return n;
  },
};
