module.exports = {
  null_marks: (rows) => rows.filter(([, v]) => v === -1).length,
};
