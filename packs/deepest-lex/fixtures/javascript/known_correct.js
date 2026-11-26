module.exports = {
  longest_word: (words) => words
    .split(",")
    .filter((w) => w.length > 0)
    .reduce((best, w) => Math.max(best, w.length), 0),
};
