module.exports = {
  edit_toll: (first, second) => {
    const n = first.length;
    const m = second.length;
    const d = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));
    for (let i = 0; i <= n; i += 1) d[i][0] = i;
    for (let j = 0; j <= m; j += 1) d[0][j] = j;
    for (let i = 1; i <= n; i += 1) {
      for (let j = 1; j <= m; j += 1) {
        const cost = first[i - 1] === second[j - 1] ? 0 : 1;
        d[i][j] = Math.min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + cost);
      }
    }
    return d[n][m];
  },
};
