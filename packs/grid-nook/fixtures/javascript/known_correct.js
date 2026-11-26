module.exports = {
  nook_size: (grid) => {
    const rows = grid.length;
    const cols = grid[0].length;
    if (grid[0][0] === 0) return 0;
    const seen = new Set(['0,0']);
    const stack = [[0, 0]];
    while (stack.length) {
      const [r, c] = stack.pop();
      for (const [dr, dc] of [[0, 1], [0, -1], [1, 0], [-1, 0]]) {
        const nr = r + dr;
        const nc = c + dc;
        const key = `${nr},${nc}`;
        if (nr >= 0 && nr < rows && nc >= 0 && nc < cols && grid[nr][nc] === 1 && !seen.has(key)) {
          seen.add(key);
          stack.push([nr, nc]);
        }
      }
    }
    return seen.size;
  },
};
