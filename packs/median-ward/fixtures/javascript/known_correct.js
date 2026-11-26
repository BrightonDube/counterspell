module.exports = {
  median_x2: (marks) => {
    const m = [...marks].sort((a, b) => a - b);
    const n = m.length;
    if (n === 0) return 0;
    if (n % 2) return 2 * m[Math.floor(n / 2)];
    return m[n / 2 - 1] + m[n / 2];
  },
};
