module.exports = {
  slate_len: (marks) => {
    if (marks.length === 0) return 0;
    const d = new Array(marks.length).fill(1);
    for (let i = 0; i < marks.length; i += 1) {
      for (let j = 0; j < i; j += 1) {
        if (marks[j] < marks[i]) d[i] = Math.max(d[i], d[j] + 1);
      }
    }
    return Math.max(...d);
  },
};
