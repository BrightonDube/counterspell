module.exports = {
  mod_toll: (values, m) => {
    let r = 0;
    for (const v of values) r += v;
    return ((r % m) + m) % m;
  },
};
