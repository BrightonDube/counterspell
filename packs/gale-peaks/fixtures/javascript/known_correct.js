module.exports = {
  gale_peaks: (readings, width) => {
    const peaks = [];
    const candidates = [];
    for (let i = 0; i < readings.length; i += 1) {
      while (candidates.length && readings[candidates[candidates.length - 1]] <= readings[i]) {
        candidates.pop();
      }
      candidates.push(i);
      if (candidates[0] <= i - width) {
        candidates.shift();
      }
      if (i >= width - 1) {
        peaks.push(readings[candidates[0]]);
      }
    }
    return peaks;
  },
};
