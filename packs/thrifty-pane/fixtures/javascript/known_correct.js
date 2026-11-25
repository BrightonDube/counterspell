module.exports = {
  longest_thrifty: (pane) => {
    const counts = {};
    let left = 0;
    let best = 0;
    for (let right = 0; right < pane.length; right += 1) {
      const rune = pane[right];
      counts[rune] = (counts[rune] || 0) + 1;
      while (Object.keys(counts).length > 2) {
        const edge = pane[left];
        counts[edge] -= 1;
        if (counts[edge] === 0) {
          delete counts[edge];
        }
        left += 1;
      }
      best = Math.max(best, right - left + 1);
    }
    return best;
  },
};
