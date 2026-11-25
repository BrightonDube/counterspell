module.exports = {
  shortest_varied: (inscription) => {
    const counts = { a: 0, b: 0, c: 0 };
    let held = 0;
    let left = 0;
    let best = inscription.length + 1;
    for (let right = 0; right < inscription.length; right += 1) {
      const rune = inscription[right];
      if (rune in counts) {
        if (counts[rune] === 0) {
          held += 1;
        }
        counts[rune] += 1;
      }
      while (held === 3) {
        best = Math.min(best, right - left + 1);
        const edge = inscription[left];
        if (edge in counts) {
          counts[edge] -= 1;
          if (counts[edge] === 0) {
            held -= 1;
          }
        }
        left += 1;
      }
    }
    return best > inscription.length ? 0 : best;
  },
};
