module.exports = {
  lock_order: (wards, lens) => {
    const seen = new Set();
    let at = 0;
    for (const count of lens) {
      const row = wards.slice(at, at + count);
      at += count;
      for (let i = 0; i < row.length; i++) {
        for (let j = i + 1; j < row.length; j++) {
          const a = row[i], b = row[j];
          if (a !== b && seen.has(b + ":" + a)) return 0;
          seen.add(a + ":" + b);
        }
      }
    }
    return 1;
  },
};
