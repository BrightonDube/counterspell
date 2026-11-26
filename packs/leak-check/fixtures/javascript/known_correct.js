module.exports = {
  leak_check: (train, test) => {
    const tr = new Set(train);
    return new Set(test.filter((x) => tr.has(x))).size;
  },
};
