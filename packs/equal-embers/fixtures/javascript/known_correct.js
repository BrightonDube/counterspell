module.exports = {
  balanced_span: (embers) => {
    const firstSeen = new Map([[0, -1]]);
    let balance = 0;
    let longest = 0;
    embers.forEach((ember, index) => {
      balance += ember === 1 ? 1 : -1;
      if (firstSeen.has(balance)) {
        longest = Math.max(longest, index - firstSeen.get(balance));
      } else {
        firstSeen.set(balance, index);
      }
    });
    return longest;
  },
};
