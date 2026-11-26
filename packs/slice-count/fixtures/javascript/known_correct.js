module.exports = {
  slice_count: (bursts, q) => bursts.reduce((t, b) => t + Math.ceil(b / q), 0),
};
