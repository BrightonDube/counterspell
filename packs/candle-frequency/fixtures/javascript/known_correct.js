module.exports = {
  top_burn: (sizes) => {
    const counts = new Map();
    for (const size of sizes) counts.set(size, (counts.get(size) ?? 0) + 1);
    let best = 0;
    let bestCount = 0;
    for (const [size, count] of counts) {
      if (count > bestCount || (count === bestCount && size < best)) {
        best = size;
        bestCount = count;
      }
    }
    return best;
  },
};
