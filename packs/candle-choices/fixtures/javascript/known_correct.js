module.exports = {
  pick_ways: (n, k) => {
    const f = (n, k) => {
      if (k < 0 || k > n) return 0;
      if (k === 0) return 1;
      return f(n - 1, k - 1) + f(n - 1, k);
    };
    return f(n, k);
  },
};
