module.exports = {
  header_check: (env) => {
    const [v, s, q, l] = env;
    const ok = (v === 1 || v === 2) && s >= 0 && s <= 255 && q >= 0 && q <= 65535 && l >= 0 && l <= 1500;
    return ok ? 1 : 0;
  },
};
