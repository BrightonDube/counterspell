module.exports = {
  decode_ways: (glyphs) => {
    const n = glyphs.length;
    if (n === 0) return 1;
    const d = new Array(n + 1).fill(0);
    d[0] = 1;
    for (let i = 1; i <= n; i += 1) {
      if (glyphs[i - 1] !== "0") d[i] += d[i - 1];
      if (i >= 2) {
        const pair = Number(glyphs.slice(i - 2, i));
        if (pair >= 10 && pair <= 26) d[i] += d[i - 2];
      }
    }
    return d[n];
  },
};
