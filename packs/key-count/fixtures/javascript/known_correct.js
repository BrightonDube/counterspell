module.exports = {
  key_count: (rows) => new Set(rows.map(([k]) => k)).size,
};
