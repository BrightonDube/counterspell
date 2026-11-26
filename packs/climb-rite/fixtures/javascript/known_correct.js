module.exports = {
  climb_ways: (steps) => {
    if (steps <= 0) return 1;
    let a = 1;
    let b = 1;
    for (let i = 0; i < steps - 1; i += 1) {
      [a, b] = [b, a + b];
    }
    return b;
  },
};
