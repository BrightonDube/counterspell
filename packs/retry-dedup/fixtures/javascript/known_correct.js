module.exports = {
  retry_dedup: (log) => new Set(log.map(([, r]) => r)).size,
};
