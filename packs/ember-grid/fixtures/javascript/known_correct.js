module.exports = {
  grid_min: (grid) => {
    const h = grid.length;
    const w = grid[0].length;
    const d = Array.from({ length: h }, () => new Array(w).fill(0));
    for (let i = 0; i < h; i += 1) {
      for (let j = 0; j < w; j += 1) {
        if (i === 0 && j === 0) {
          d[i][j] = grid[0][0];
        } else {
          const best = Math.min(i > 0 ? d[i - 1][j] : Infinity, j > 0 ? d[i][j - 1] : Infinity);
          d[i][j] = best + grid[i][j];
        }
      }
    }
    return d[h - 1][w - 1];
  },
};
