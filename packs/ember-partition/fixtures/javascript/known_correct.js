module.exports = {
  partition_count: (marks, ways) => {
    const total = marks.reduce((a, b) => a + b, 0);
    if (total % ways || marks.length < ways) return 0;
    const target = total / ways;
    const sums = new Array(ways).fill(0);
    const counts = new Array(ways).fill(0);
    const go = (i) => {
      if (i === marks.length) return counts.every((c) => c > 0) ? 1 : 0;
      let placed = 0;
      for (let g = 0; g < ways; g += 1) {
        if (counts[g] === 0 && g > 0 && counts[g - 1] === 0) break;
        if (sums[g] + marks[i] > target) continue;
        sums[g] += marks[i];
        counts[g] += 1;
        placed += go(i + 1);
        sums[g] -= marks[i];
        counts[g] -= 1;
        if (counts[g] === 0) break;
      }
      return placed;
    };
    return go(0);
  },
};
