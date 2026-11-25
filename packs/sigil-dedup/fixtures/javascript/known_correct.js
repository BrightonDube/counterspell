module.exports = {
  distinct_sigils: (marks) => {
    let distinct = 0;
    let previous = null;
    for (const mark of marks) {
      if (mark !== previous) {
        distinct += 1;
        previous = mark;
      }
    }
    return distinct;
  },
};
