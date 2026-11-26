module.exports = {
  serial_check: (events) => {
    const done = new Set();
    let prev = null;
    for (const t of events) {
      if (done.has(t)) return 0;
      if (t !== prev) {
        if (prev !== null) done.add(prev);
        prev = t;
      }
    }
    return 1;
  },
};
