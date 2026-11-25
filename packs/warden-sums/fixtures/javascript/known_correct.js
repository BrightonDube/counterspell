module.exports = {
  ritual_spans: (readings, quota) => {
    const seen = new Map([[0, 1]]);
    let total = 0;
    let spans = 0;
    for (const reading of readings) {
      total += reading;
      spans += seen.get(total - quota) ?? 0;
      seen.set(total, (seen.get(total) ?? 0) + 1);
    }
    return spans;
  },
};
