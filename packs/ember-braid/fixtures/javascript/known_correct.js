module.exports = {
  braid_ways: (text, first, second) => {
    const n = first.length;
    const m = second.length;
    if (text.length !== n + m) return 0;
    const d = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));
    d[0][0] = 1;
    for (let i = 0; i <= n; i += 1) {
      for (let j = 0; j <= m; j += 1) {
        if (i > 0 && first[i - 1] === text[i + j - 1]) d[i][j] |= d[i - 1][j];
        if (j > 0 && second[j - 1] === text[i + j - 1]) d[i][j] |= d[i][j - 1];
      }
    }
    return d[n][m];
  },
};
