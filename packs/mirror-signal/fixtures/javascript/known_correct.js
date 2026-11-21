module.exports = {
  mirror_signal: (signal) => {
    let mismatches = 0;
    for (let left = 0; left < Math.floor(signal.length / 2); left += 1) {
      const right = signal.length - left - 1;
      if (signal[left] !== signal[right]) {
        mismatches += 1;
      }
    }
    return mismatches;
  },
};
