module.exports = {
  lone_count: (edges, n) => {
    const touched = new Array(Math.max(0, n)).fill(false);
    for (const [a, b] of edges) {
      if (a >= 0 && a < n) touched[a] = true;
      if (b >= 0 && b < n) touched[b] = true;
    }
    return touched.filter((t) => !t).length;
  },
};
