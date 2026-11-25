module.exports = {
  tail_mark: (marks, nxt) => {
    let i = nxt.length ? 0 : -1, last = -1;
    while (i !== -1) { last = i; i = nxt[i]; }
    return last === -1 ? -1 : marks[last];
  },
};
