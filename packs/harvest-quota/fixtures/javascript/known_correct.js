module.exports = {
  longest_harvest: (marks, quota) => {
    let left = 0;
    let total = 0;
    let best = 0;
    for (let right = 0; right < marks.length; right += 1) {
      total += marks[right];
      while (total > quota) {
        total -= marks[left];
        left += 1;
      }
      best = Math.max(best, right - left + 1);
    }
    return best;
  },
};
