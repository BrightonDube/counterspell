module.exports = {
  cross_pairs: (rows) => {
    let total = 0;
    let ahead = rows.reduce((sum, count) => sum + count, 0);
    for (const count of rows) {
      ahead -= count;
      total += count * ahead;
    }
    return total;
  },
};
