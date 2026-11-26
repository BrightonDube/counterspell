module.exports = {
  norm_rite: (v) => {
    const s = v.reduce((t, x) => t + x, 0);
    if (s <= 0) return 0;
    let shares = 0;
    for (const x of v) shares += Math.floor((x * 100) / s);
    return 100 - shares;
  },
};
