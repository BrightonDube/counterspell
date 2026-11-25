module.exports = {
  warmest_window: (readings, width) => {
    if (readings.length < width) {
      return 0;
    }
    let total = 0;
    for (let i = 0; i < width; i += 1) {
      total += readings[i];
    }
    let best = total;
    for (let i = width; i < readings.length; i += 1) {
      total += readings[i] - readings[i - width];
      if (total > best) {
        best = total;
      }
    }
    return best;
  },
};
