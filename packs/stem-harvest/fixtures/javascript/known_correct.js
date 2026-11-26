module.exports = {
  stem_depth_sum: (words) => {
    const ws = new Set(words.split(",").filter((w) => w.length > 0));
    let ans = 0;
    for (const w of ws) {
      let best = 0;
      for (const o of ws) {
        if (o !== w && w.startsWith(o) && o.length > best) best = o.length;
      }
      ans += best;
    }
    return ans;
  },
};
