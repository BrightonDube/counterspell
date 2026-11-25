module.exports = {
  drift_alarms: (readings, width) => {
    let alarms = 0;
    for (let i = width; i < readings.length; i += 1) {
      let peak = readings[i - width];
      for (let j = i - width + 1; j < i; j += 1) {
        if (readings[j] > peak) {
          peak = readings[j];
        }
      }
      if (readings[i] > peak) {
        alarms += 1;
      }
    }
    return alarms;
  },
};
