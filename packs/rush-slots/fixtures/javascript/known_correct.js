module.exports = {
  pick_count: (spans) => {
    let n = 0;
    let end = -1;
    for (const [a, b] of [...spans].sort((x, y) => x[1] - y[1] || x[0] - y[0])) {
      if (a > end) {
        n += 1;
        end = b;
      }
    }
    return n;
  },
};
