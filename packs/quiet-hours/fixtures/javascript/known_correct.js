module.exports = {
  quiet_windows: (readings, quota) => {
    let quiet = 0;
    for (let i = 0; i + 2 < readings.length; i += 1) {
      if (readings[i] + readings[i + 1] + readings[i + 2] <= quota) {
        quiet += 1;
      }
    }
    return quiet;
  },
};
