module.exports = {
  row_join: (a, b) => {
    let count = 0;
    for (const x of a) {
      for (const y of b) {
        if (x[0] === y[0]) count += 1;
      }
    }
    return count;
  },
};
