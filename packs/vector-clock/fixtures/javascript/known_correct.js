module.exports = {
  clock_cmp: (a, b) => {
    let less = false, more = false;
    for (let i = 0; i < a.length; i++) {
      if (a[i] < b[i]) less = true;
      if (a[i] > b[i]) more = true;
    }
    if (less && more) return 2;
    if (less) return -1;
    if (more) return 1;
    return 0;
  },
};
