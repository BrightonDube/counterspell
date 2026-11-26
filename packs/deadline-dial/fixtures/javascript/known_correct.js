module.exports = {
  dial_count: (slots) => {
    const jobs = [];
    for (let i = 0; i < slots.length; i += 2) {
      jobs.push([slots[i], slots[i + 1]]);
    }
    jobs.sort((a, b) => a[0] - b[0]);
    const held = [];
    let time = 0;
    for (const [d, t] of jobs) {
      time += t;
      held.push(t);
      held.sort((a, b) => b - a);
      if (time > d) {
        time -= held.shift();
      }
    }
    return held.length;
  },
};
