module.exports = {
  top_sigil: (ops) => {
    const marks = [];
    for (const op of ops) {
      if (op === 0) {
        marks.pop();
      } else {
        marks.push(op);
      }
    }
    return marks.length === 0 ? -1 : marks[marks.length - 1];
  },
};
