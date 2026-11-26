module.exports = {
  bit_isolate: (n) => (n === 0 ? 0 : n & (-n)),
};
