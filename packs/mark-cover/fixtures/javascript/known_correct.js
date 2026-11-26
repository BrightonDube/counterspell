module.exports = {
  cover_length: (spans) => {
    const cov = new Array(101).fill(false);
    for (const [a, b] of spans) {
      for (let i = a; i <= b; i += 1) cov[i] = true;
    }
    return cov.filter(Boolean).length;
  },
};
