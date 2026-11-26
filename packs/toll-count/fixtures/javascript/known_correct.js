module.exports = {
  toll_count: (edges, weights, t) => {
    let count = 0;
    for (let k = 0; k < Math.min(edges.length, weights.length); k += 1) {
      if (weights[k] <= t) count += 1;
    }
    return count;
  },
};
