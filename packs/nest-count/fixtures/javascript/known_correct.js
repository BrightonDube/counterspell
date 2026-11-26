module.exports = {
  nest_pairs: (spans) => {
    let n = 0;
    for (let i = 0; i < spans.length; i += 1) {
      for (let j = i + 1; j < spans.length; j += 1) {
        const [a, b] = spans[i];
        const [c, d] = spans[j];
        if ((a < c && d < b) || (c < a && b < d)) n += 1;
      }
    }
    return n;
  },
};
