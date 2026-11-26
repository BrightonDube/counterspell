module.exports = {
  reverse_order: (marks, nxt) => {
    const out = [];
    for (let i = nxt.length ? 0 : -1; i !== -1; i = nxt[i]) { out.push(marks[i]); }
    return out.reverse();
  },
};
