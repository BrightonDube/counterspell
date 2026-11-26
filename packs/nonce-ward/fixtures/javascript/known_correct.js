module.exports = {
  nonce_ward: (msgs) => {
    const high = new Map();
    let replays = 0;
    for (const [s, n] of msgs) {
      const mark = high.has(s) ? high.get(s) : -1;
      if (n <= mark) replays++; else high.set(s, n);
    }
    return replays;
  },
};
