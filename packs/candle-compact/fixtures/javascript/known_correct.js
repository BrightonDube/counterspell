module.exports = {
  remaining_candles: (marks, cursed) =>
    marks.filter((mark) => mark !== cursed).length,
};
