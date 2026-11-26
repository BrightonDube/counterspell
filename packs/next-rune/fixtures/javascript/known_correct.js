module.exports = {
  next_letters: (words, prefix) => {
    const nxts = new Set();
    for (const w of words.split(",")) {
      if (w && w.startsWith(prefix) && w.length > prefix.length) {
        nxts.add(w[prefix.length]);
      }
    }
    return nxts.size;
  },
};
