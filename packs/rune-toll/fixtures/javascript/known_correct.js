module.exports = {
  toll_sum: (edges, weights, n) => {
    let total = 0;
    for (let k = 0; k < Math.min(edges.length, weights.length); k += 1) {
      total += weights[k];
    }
    return total;
  },
};
