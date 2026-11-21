module.exports = {
  lantern_ledger: (changes, start, level) => {
    let meter = start;
    let marks = 0;
    for (const change of changes) {
      meter += change;
      if (meter === level) {
        marks += 1;
      }
    }
    return marks;
  },
};
