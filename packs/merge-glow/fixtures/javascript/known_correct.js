module.exports = {
  merge_glow: (am, an, bm, bn) => {
    const out = [];
    let i = an.length ? 0 : -1, j = bn.length ? 0 : -1;
    while (i !== -1 && j !== -1) {
      if (am[i] <= bm[j]) { out.push(am[i]); i = an[i]; }
      else { out.push(bm[j]); j = bn[j]; }
    }
    for (; i !== -1; i = an[i]) { out.push(am[i]); }
    for (; j !== -1; j = bn[j]) { out.push(bm[j]); }
    return out;
  },
};
