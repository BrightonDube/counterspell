module.exports = {
  meld: (a, b) => {
    const out = [];
    let i = 0, j = 0;
    while (i < a.length && j < b.length) {
      if (a[i] <= b[j]) { out.push(a[i]); i += 1; }
      else { out.push(b[j]); j += 1; }
    }
    return out.concat(a.slice(i), b.slice(j));
  },
};
