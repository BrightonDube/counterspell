module.exports = {
  can_summon: (marks, target) => {
    const go = (i, need) =>
      need === 0 || (i < marks.length && (go(i + 1, need - marks[i]) || go(i + 1, need)));
    return go(0, target) ? 1 : 0;
  },
};
