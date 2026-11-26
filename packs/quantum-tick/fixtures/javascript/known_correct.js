module.exports = {
  quantum_tick: (jobs, q) => {
    const order = jobs.map((_, i) => i).sort((x, y) => jobs[x][0] - jobs[y][0] || x - y);
    const rem = jobs.map(j => j[1]);
    const ready = [];
    let i = 0, t = 0, total = 0, done = 0;
    const n = order.length;
    while (done < n) {
      while (i < n && jobs[order[i]][0] <= t) { ready.push(order[i]); i += 1; }
      if (ready.length === 0) { t = jobs[order[i]][0]; continue; }
      const j = ready.shift();
      const run = Math.min(q, rem[j]);
      t += run;
      rem[j] -= run;
      while (i < n && jobs[order[i]][0] <= t) { ready.push(order[i]); i += 1; }
      if (rem[j] > 0) ready.push(j);
      else { total += t; done += 1; }
    }
    return total;
  },
};
