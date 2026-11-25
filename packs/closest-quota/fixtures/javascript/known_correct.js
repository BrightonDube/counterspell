module.exports = {
  nearest_pair: (marks, quota) => {
    if (marks.length < 2) return 0;
    let left = 0;
    let right = marks.length - 1;
    let best = null;
    while (left < right) {
      const total = marks[left] + marks[right];
      if (
        best === null ||
        Math.abs(total - quota) < Math.abs(best - quota) ||
        (Math.abs(total - quota) === Math.abs(best - quota) && total < best)
      ) {
        best = total;
      }
      if (total < quota) left += 1;
      else right -= 1;
    }
    return best;
  },
};
