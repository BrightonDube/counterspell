module.exports = {
  miss_count: (marks, c) => {
    const slots = new Array(c).fill(-1);
    let misses = 0;
    for (const m of marks) {
      const line = m % c, tag = Math.floor(m / c);
      if (slots[line] !== tag) { misses += 1; slots[line] = tag; }
    }
    return misses;
  },
};
