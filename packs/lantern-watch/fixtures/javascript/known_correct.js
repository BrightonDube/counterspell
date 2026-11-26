module.exports = {
  watch_points: (spans) => {
    const s = [...spans].filter(([a, b]) => a <= b).sort((x, y) => x[1] - y[1] || x[0] - y[0]);
    let pts = 0;
    let i = 0;
    while (i < s.length) {
      const p = s[i][1];
      pts += 1;
      while (i < s.length && s[i][0] <= p) i += 1;
    }
    return pts;
  },
};
