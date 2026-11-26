module.exports = {
  path_count: (grid) => {
    const rows = grid.length;
    const cols = grid[0].length;
    const f = (r, c) => {
      if (r >= rows || c >= cols || grid[r][c] === 0) return 0;
      if (r === rows - 1 && c === cols - 1) return 1;
      return f(r, c + 1) + f(r + 1, c);
    };
    return f(0, 0);
  },
};
