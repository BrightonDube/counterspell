module.exports = {
  toll_stairs: (costs) => {
    const n = costs.length;
    if (n <= 1) return n === 1 ? costs[0] : 0;
    const d = new Array(n + 1).fill(0);
    for (let i = 2; i <= n; i += 1) {
      d[i] = Math.min(d[i - 1] + costs[i - 1], d[i - 2] + costs[i - 2]);
    }
    return d[n];
  },
};
