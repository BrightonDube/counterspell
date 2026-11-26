module.exports = {
  chain_length: (nxt) => {
    let i = nxt.length ? 0 : -1, count = 0;
    while (i !== -1) { count += 1; i = nxt[i]; }
    return count;
  },
};
