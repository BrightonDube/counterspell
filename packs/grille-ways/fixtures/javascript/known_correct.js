module.exports = {
  grille_ways: (rows, cols) => {
    if (rows <= 0 || cols <= 0) return 0;
    const d = Array.from({ length: rows }, () => new Array(cols).fill(0));
    for (let i = 0; i < rows; i += 1) {
      for (let j = 0; j < cols; j += 1) {
        d[i][j] = i === 0 || j === 0 ? 1 : d[i - 1][j] + d[i][j - 1];
      }
    }
    return d[rows - 1][cols - 1];
  },
};
