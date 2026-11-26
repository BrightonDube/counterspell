module.exports = {
  mask_rite: (weights, cap) => {
    const n = weights.length;
    let count = 0;
    for (let mask = 0; mask < (1 << n); mask += 1) {
      let total = 0;
      for (let i = 0; i < n; i += 1) {
        if ((mask >> i) & 1) total += weights[i];
      }
      if (total <= cap) count += 1;
    }
    return count;
  },
};
