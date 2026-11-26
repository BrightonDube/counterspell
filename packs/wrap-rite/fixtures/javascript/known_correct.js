module.exports = {
  wrap_rite: (marks, b) => {
    const ring = [];
    let evictions = 0;
    for (const m of marks) {
      if (!ring.includes(m)) {
        if (ring.length === b) { ring.shift(); evictions += 1; }
        ring.push(m);
      }
    }
    return evictions;
  },
};
