module.exports = {
  chunk_toll: (tokens, w, base) => {
    const n = tokens.length;
    const width = Math.max(w, 1);
    const dp = new Array(n + 1).fill(Number.MAX_SAFE_INTEGER);
    dp[0] = 0;
    for (let i = 1; i <= n; i++) {
      for (let j = Math.max(0, i - width); j < i; j++) {
        const c = tokens.slice(j, i);
        const cost = dp[j] + base + (Math.max(...c) - Math.min(...c));
        if (cost < dp[i]) dp[i] = cost;
      }
    }
    return dp[n];
  },
};
