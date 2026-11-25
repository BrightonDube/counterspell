module.exports = {
  shared_link: (nxt, ha, hb) => {
    const seen = new Set();
    for (let i = ha; i !== -1; i = nxt[i]) { seen.add(i); }
    let j = hb;
    while (j !== -1 && !seen.has(j)) { j = nxt[j]; }
    return j;
  },
};
