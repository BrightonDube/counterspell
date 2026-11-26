module.exports = {
  cover_pick: (n, masks) => {
    const full = (1 << n) - 1;
    let covered = 0, picks = 0;
    const pop = (x) => { let c = 0; while (x) { c += x & 1; x >>>= 1; } return c; };
    while (covered !== full) {
      let gain = 0, best = -1;
      for (let i = 0; i < masks.length; i++) {
        const g = pop(masks[i] & full & ~covered);
        if (g > gain) { gain = g; best = i; }
      }
      if (best < 0) return picks;
      covered |= masks[best] & full;
      picks++;
    }
    return picks;
  },
};
