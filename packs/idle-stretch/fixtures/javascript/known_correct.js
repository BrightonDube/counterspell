module.exports = {
  idle_stretch: (spans, t) => {
    const cov = new Array(t + 1).fill(false);
    for (const [a, b] of spans) {
      for (let i = Math.max(a, 0); i <= Math.min(b, t); i += 1) cov[i] = true;
    }
    let best = 0;
    let run = 0;
    for (let i = 0; i <= t; i += 1) {
      run = cov[i] ? 0 : run + 1;
      if (run > best) best = run;
    }
    return best;
  },
};
