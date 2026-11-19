module.exports = {
  longest_run: (text) => {
    let best = 0;
    let streak = 0;
    let last = null;
    for (const char of text) {
      streak = char === last ? streak + 1 : 1;
      last = char;
      best = Math.max(best, streak);
    }
    return best;
  },
};
