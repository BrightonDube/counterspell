module.exports = {
  floor_sum: (marks, k) =>
    [...marks].sort((a, b) => a - b).slice(0, k).reduce((a, b) => a + b, 0),
};
