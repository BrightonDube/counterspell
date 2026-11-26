module.exports = {
  burst_rite: (jobs) => {
    const order = jobs.map((_, i) => i).sort((x, y) => jobs[x][0] - jobs[y][0] || x - y);
    let clock = 0, total = 0;
    for (const i of order) {
      const [a, b] = jobs[i];
      const start = Math.max(clock, a);
      total += start - a;
      clock = start + b;
    }
    return total;
  },
};
