module.exports = {
  block_count: (spans) => {
    if (spans.length === 0) return 0;
    const s = [...spans].sort((x, y) => x[0] - y[0] || x[1] - y[1]);
    let blocks = 1;
    let end = s[0][1];
    for (const [a, b] of s.slice(1)) {
      if (a > end) {
        blocks += 1;
        end = b;
      } else {
        end = Math.max(end, b);
      }
    }
    return blocks;
  },
};
