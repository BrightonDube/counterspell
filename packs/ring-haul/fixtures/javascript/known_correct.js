module.exports = {
  ring_haul: (marks) => {
    const line = (ms) => {
      let take = 0;
      let skip = 0;
      for (const m of ms) {
        const t = skip + m;
        skip = Math.max(skip, take);
        take = t;
      }
      return Math.max(take, skip);
    };
    if (marks.length <= 1) return marks.length === 1 ? marks[0] : 0;
    return Math.max(line(marks.slice(1)), line(marks.slice(0, -1)));
  },
};
