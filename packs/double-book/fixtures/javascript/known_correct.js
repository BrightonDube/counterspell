module.exports = {
  room_count: (spans) => {
    const evs = [];
    for (const [a, b] of spans) {
      evs.push([a, 1]);
      evs.push([b, -1]);
    }
    evs.sort((x, y) => x[0] - y[0] || y[1] - x[1]);
    let cur = 0;
    let best = 0;
    for (const [, d] of evs) {
      cur += d;
      if (cur > best) best = cur;
    }
    return best;
  },
};
