module.exports = {
  near_mark: (edges, weights, n) => {
    let best = null;
    for (let k = 0; k < Math.min(edges.length, weights.length); k += 1) {
      const [a, b] = edges[k];
      const c = weights[k];
      if (a === 0 || b === 0) {
        if (best === null || c < best) best = c;
      }
    }
    return best === null ? -1 : best;
  },
};
