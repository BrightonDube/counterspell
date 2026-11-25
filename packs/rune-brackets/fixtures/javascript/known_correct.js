module.exports = {
  nested_runes: (inscription) => {
    const partners = { ")": "(", "]": "[", "}": "{" };
    const openers = [];
    for (const rune of inscription) {
      if (rune in partners) {
        if (openers.length === 0 || openers[openers.length - 1] !== partners[rune]) {
          return 0;
        }
        openers.pop();
      } else {
        openers.push(rune);
      }
    }
    return openers.length === 0 ? 1 : 0;
  },
};
