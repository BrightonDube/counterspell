module.exports = {
  diagonal_total: (grid) => {
    const edge = Math.min(grid.length, grid.length ? grid[0].length : 0);
    let total = 0;
    for (let row = 0; row < edge; row += 1) {
      total += grid[row][row];
    }
    return total;
  },
};
