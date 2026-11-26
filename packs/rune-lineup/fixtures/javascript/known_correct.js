module.exports = {
  lineups: (marks) => {
    const go = (prev, left) => {
      if (left.length === 0) return 1;
      let total = 0;
      for (let i = 0; i < left.length; i += 1) {
        if (left[i] !== prev) {
          total += go(left[i], left.slice(0, i).concat(left.slice(i + 1)));
        }
      }
      return total;
    };
    return go(null, marks);
  },
};
