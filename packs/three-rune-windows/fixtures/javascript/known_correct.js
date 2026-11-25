module.exports = {
  count_varied: (inscription) => {
    const n = inscription.length;
    let total = 0;
    for (let left = 0; left < n; left += 1) {
      const missing = new Set(["a", "b", "c"]);
      for (let right = left; right < n; right += 1) {
        missing.delete(inscription[right]);
        if (missing.size === 0) {
          total += n - right;
          break;
        }
      }
    }
    return total;
  },
};
