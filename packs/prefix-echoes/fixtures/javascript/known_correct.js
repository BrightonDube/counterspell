module.exports = {
  prefix_counts: (marks, queries) =>
    queries.map(
      ([k, v]) => marks.slice(0, k).filter((mark) => mark === v).length,
    ),
};
