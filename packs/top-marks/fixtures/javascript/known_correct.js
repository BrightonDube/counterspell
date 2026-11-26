module.exports = {
  top_marks: (scores, k) => {
    const s = [...scores].sort((x, y) => y - x).slice(0, k);
    return s.reduce((t, v) => t + v, 0);
  },
};
