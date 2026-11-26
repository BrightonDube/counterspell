module.exports = {
  rate_gate: (events, capacity, refill) => {
    let bucket = capacity, last = 0, passed = 0;
    for (const [tick, take] of events) {
      bucket = Math.min(capacity, bucket + (tick - last) * refill);
      last = tick;
      if (take <= bucket) { bucket -= take; passed++; }
    }
    return passed;
  },
};
