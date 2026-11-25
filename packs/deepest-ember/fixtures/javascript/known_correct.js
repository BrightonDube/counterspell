module.exports = {
  deepest_ember: (inscription) => {
    let depth = 0;
    let deepest = 0;
    for (const rune of inscription) {
      if (rune === "(") {
        depth += 1;
        deepest = Math.max(deepest, depth);
      } else if (depth > 0) {
        depth -= 1;
      }
    }
    return deepest;
  },
};
