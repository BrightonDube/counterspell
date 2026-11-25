module.exports = {
  longest_calm: (chant) => {
    let best = 0;
    let current = 0;
    for (const rune of chant) {
      if (rune === "#") {
        current = 0;
      } else {
        current += 1;
        if (current > best) {
          best = current;
        }
      }
    }
    return best;
  },
};
