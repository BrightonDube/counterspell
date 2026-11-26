module.exports = {
  spread_ways: (cells, guards) => {
    const f = (c, g) => {
      if (g === 0) return 1;
      if (c <= 0) return 0;
      return f(c - 2, g - 1) + f(c - 1, g);
    };
    return f(cells, guards);
  },
};
