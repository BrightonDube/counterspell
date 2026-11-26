module.exports = {
  stale_count: (seen, queued) =>
    seen.reduce((n, a, i) => n + (a !== queued[i] ? 1 : 0), 0),
};
