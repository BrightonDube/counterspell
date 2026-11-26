module.exports = {
  relay_count: (spans, gap) => {
    let n = 0;
    let end = -1000000000;
    for (const [a, b] of [...spans].sort((x, y) => x[1] - y[1] || x[0] - y[0])) {
      if (a >= end + gap) {
        n += 1;
        end = b;
      }
    }
    return n;
  },
};
