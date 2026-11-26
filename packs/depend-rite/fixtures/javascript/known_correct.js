module.exports = {
  depend_rite: (rows) => {
    const seen = new Map();
    for (const [x, y] of rows) {
      if (seen.has(x) && seen.get(x) !== y) return 0;
      seen.set(x, y);
    }
    return 1;
  },
};
