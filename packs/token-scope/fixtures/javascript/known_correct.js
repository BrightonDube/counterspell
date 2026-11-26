module.exports = {
  scope_ok: (required, granted, expiry, now) => {
    const alive = now < expiry;
    const covered = required.every((s) => granted.includes(s));
    return alive && covered ? 1 : 0;
  },
};
