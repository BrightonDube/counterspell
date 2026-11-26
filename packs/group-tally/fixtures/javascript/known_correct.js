module.exports = {
  group_tally: (rows) => {
    const tally = new Map();
    for (const [k, v] of rows) {
      tally.set(k, (tally.get(k) || 0) + v);
    }
    return tally.size ? Math.max(...tally.values()) : 0;
  },
};
