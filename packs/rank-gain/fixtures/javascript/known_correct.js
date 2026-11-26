module.exports = {
  rank_gain: (rels, k) => {
    let total = 0;
    for (let i = 0; i < Math.min(k, rels.length); i++) total += rels[i] * (k - i);
    return total;
  },
};
