module.exports = {
  largest_sigil: (markings) => {
    const groups = new Map();
    for (const marking of markings) {
      const sigil = String(marking)
        .split("")
        .reduce((total, digit) => total + Number(digit), 0);
      groups.set(sigil, (groups.get(sigil) ?? 0) + 1);
    }
    return groups.size === 0 ? 0 : Math.max(...groups.values());
  },
};
