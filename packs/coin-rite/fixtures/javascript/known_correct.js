module.exports = {
  coin_rite: (coins, target) => {
    const d = new Array(target + 1).fill(Infinity);
    d[0] = 0;
    for (let i = 1; i <= target; i += 1) {
      for (const c of coins) {
        if (i - c >= 0) d[i] = Math.min(d[i], d[i - c] + 1);
      }
    }
    return d[target] === Infinity ? -1 : d[target];
  },
};
