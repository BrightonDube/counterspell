module.exports = {
  dot_sparse: (a, b) => {
    const da = new Map(a.map(([p, m]) => [p, m]));
    const db = new Map(b.map(([p, m]) => [p, m]));
    let total = 0;
    for (const [p, m] of da) if (db.has(p)) total += m * db.get(p);
    return total;
  },
};
