module.exports = {
  next_mark: (prios) => {
    let best = -1;
    for (let i = 0; i < prios.length; i += 1) {
      if (best < 0 || prios[i] < prios[best]) best = i;
    }
    return best;
  },
};
