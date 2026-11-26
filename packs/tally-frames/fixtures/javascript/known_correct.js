module.exports = {
  tally_frames: (m) => {
    let total = 0;
    for (let i = 0; i < m.length; i++) if (i < m[i].length) total += m[i][i];
    return total;
  },
};
