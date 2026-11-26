module.exports = {
  count_mixes: (coins, target) => {
    const d = new Array(target + 1).fill(0);
    d[0] = 1;
    for (const c of coins) {
      for (let i = c; i <= target; i += 1) {
        d[i] += d[i - c];
      }
    }
    return d[target];
  },
};
