module.exports = {
  kth_mark: (marks, k) => {
    const sorted = [...marks].sort((a, b) => a - b);
    return k > 0 && k <= sorted.length ? sorted[k - 1] : -1;
  },
};
