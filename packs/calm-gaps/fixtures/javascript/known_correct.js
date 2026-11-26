module.exports = {
  calm_haul: (marks) => {
    let take = 0;
    let skip = 0;
    for (const m of marks) {
      const t = skip + m;
      skip = Math.max(skip, take);
      take = t;
    }
    return Math.max(take, skip);
  },
};
