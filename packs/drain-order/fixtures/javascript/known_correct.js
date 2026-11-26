module.exports = {
  drain_order: (events) => {
    const pairs = [];
    for (let i = 0; i < events.length; i += 2) {
      pairs.push([events[i], events[i + 1]]);
    }
    pairs.sort((a, b) => a[0] - b[0] || a[1] - b[1]);
    return pairs.map((p) => p[1]);
  },
};
