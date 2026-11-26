module.exports = {
  prime_ward: (n) => {
    if (n < 2) return 0;
    const sieve = new Array(n + 1).fill(true);
    sieve[0] = sieve[1] = false;
    for (let p = 2; p * p <= n; p += 1) {
      if (sieve[p]) {
        for (let k = p * p; k <= n; k += p) sieve[k] = false;
      }
    }
    return sieve.filter(Boolean).length;
  },
};
