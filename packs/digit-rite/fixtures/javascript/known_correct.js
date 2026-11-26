module.exports = {
  digit_rite: (n) => {
    const orig = n;
    let count = 0;
    while (n > 0) {
      const d = n % 10;
      if (d !== 0 && orig % d === 0) count += 1;
      n = Math.floor(n / 10);
    }
    return count;
  },
};
