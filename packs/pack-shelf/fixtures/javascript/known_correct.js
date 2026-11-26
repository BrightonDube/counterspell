module.exports = {
  pack_worth: (items, cap) => {
    const d = new Array(cap + 1).fill(0);
    for (const [w, v] of items) {
      for (let i = cap; i >= w; i -= 1) {
        d[i] = Math.max(d[i], d[i - w] + v);
      }
    }
    return d[cap];
  },
};
