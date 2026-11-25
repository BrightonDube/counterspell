module.exports = {
  nth_tail: (marks, nxt, n) => {
    if (!nxt.length) { return -1; }
    let lead = 0, trail = 0;
    for (let k = 0; k < n; k += 1) {
      if (lead === -1) { return -1; }
      lead = nxt[lead];
    }
    if (lead === -1) { return -1; }
    while (nxt[lead] !== -1) { lead = nxt[lead]; trail = nxt[trail]; }
    return marks[trail];
  },
};
