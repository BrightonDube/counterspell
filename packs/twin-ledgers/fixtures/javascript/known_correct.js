module.exports = {
  matching_totals: (entries_a, entries_b) => {
    const totals = (entries) => {
      const sums = new Map();
      for (const [key, amount] of entries) {
        sums.set(key, (sums.get(key) ?? 0) + amount);
      }
      return sums;
    };
    const a = totals(entries_a);
    const b = totals(entries_b);
    let agreed = 0;
    for (const [key, sum] of a) {
      if (b.get(key) === sum) agreed += 1;
    }
    return agreed;
  },
};
