module.exports = {
  has_word: (words, word) => {
    if (word === "") return 0;
    return words.split(",").filter((w) => w === word).length > 0 ? 1 : 0;
  },
};
