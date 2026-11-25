module.exports = {
  surviving_text: (chant) => {
    const written = [];
    for (const rune of chant) {
      if (rune === "#") {
        written.pop();
      } else {
        written.push(rune);
      }
    }
    return written.join("");
  },
};
