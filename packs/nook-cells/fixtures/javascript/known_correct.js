module.exports = {
  nook_ways: (grid) => {
    const h = grid.length;
    const w = grid[0].length;
    const d = Array.from({ length: h }, () => new Array(w).fill(0));
    for (let i = 0; i < h; i += 1) {
      for (let j = 0; j < w; j += 1) {
        if (grid[i][j] === 0) continue;
        if (i === 0 && j === 0) {
          d[i][j] = 1;
        } else {
          d[i][j] = (i > 0 ? d[i - 1][j] : 0) + (j > 0 ? d[i][j - 1] : 0);
        }
      }
    }
    return d[h - 1][w - 1];
  },
};
