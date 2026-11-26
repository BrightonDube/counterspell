module.exports = {
  prefix_pairs: (words) => {
    const ws = [...new Set(words.split(",").filter((w) => w.length > 0))].sort();
    let ans = 0;
    for (let i = 0; i < ws.length; i += 1) {
      for (let j = i + 1; j < ws.length; j += 1) {
        if (ws[i] !== ws[j] && ws[j].startsWith(ws[i])) ans += 1;
      }
    }
    return ans;
  },
};
