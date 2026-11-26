module.exports = {
  word_break: (text, lexicon) => {
    const words = lexicon.split(",").filter((w) => w.length > 0);
    const n = text.length;
    const d = new Array(n + 1).fill(0);
    d[0] = 1;
    for (let i = 1; i <= n; i += 1) {
      for (const w of words) {
        if (i >= w.length && d[i - w.length] && text.slice(i - w.length, i) === w) {
          d[i] = 1;
          break;
        }
      }
    }
    return d[n];
  },
};
