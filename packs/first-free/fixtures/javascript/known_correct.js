module.exports = {
  first_free: (spans) => {
    const cov = new Set();
    for (const [a, b] of spans) {
      for (let i = a; i <= b; i += 1) cov.add(i);
    }
    let i = 0;
    while (cov.has(i)) i += 1;
    return i;
  },
};
