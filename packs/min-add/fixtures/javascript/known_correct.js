module.exports = {
  fewest_inserts: (inscription) => {
    let openLayers = 0;
    let stranded = 0;
    for (const rune of inscription) {
      if (rune === "(") {
        openLayers += 1;
      } else if (openLayers > 0) {
        openLayers -= 1;
      } else {
        stranded += 1;
      }
    }
    return openLayers + stranded;
  },
};
