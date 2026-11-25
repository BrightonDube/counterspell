module.exports = {
  loop_flag: (nxt) => {
    const seen = new Set();
    let i = nxt.length ? 0 : -1;
    while (i !== -1 && !seen.has(i)) { seen.add(i); i = nxt[i]; }
    return i !== -1 ? 1 : 0;
  },
};
