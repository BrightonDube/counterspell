module.exports = {
  echo_len: (first, second) => {
    const n = first.length;
    const m = second.length;
    const d = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));
    for (let i = 1; i <= n; i += 1) {
      for (let j = 1; j <= m; j += 1) {
        d[i][j] = first[i - 1] === second[j - 1]
          ? d[i - 1][j - 1] + 1
          : Math.max(d[i - 1][j], d[i][j - 1]);
      }
    }
    return d[n][m];
  },
};
