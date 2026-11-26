function matchWord(w, p) {
  const dp = Array.from({ length: w.length + 1 }, () =>
    new Array(p.length + 1).fill(false));
  dp[0][0] = true;
  for (let j = 1; j <= p.length; j += 1) {
    if (p[j - 1] === "*") dp[0][j] = dp[0][j - 1];
  }
  for (let i = 1; i <= w.length; i += 1) {
    for (let j = 1; j <= p.length; j += 1) {
      dp[i][j] =
        (p[j - 1] === "*" && (dp[i][j - 1] || dp[i - 1][j])) ||
        (p[j - 1] === w[i - 1] && dp[i - 1][j - 1]);
    }
  }
  return dp[w.length][p.length];
}

module.exports = {
  count_matches: (words, pattern) => {
    const ws = new Set(words.split(",").filter((w) => w.length > 0));
    let ans = 0;
    for (const w of ws) {
      if (matchWord(w, pattern)) ans += 1;
    }
    return ans;
  },
};
