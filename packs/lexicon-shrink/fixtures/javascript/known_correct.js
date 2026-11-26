module.exports = {
  shrunk_count: (words) => {
    const ws = new Set(words.split(",").filter((w) => w.length > 0));
    let ans = 0;
    for (const w of ws) {
      let absorbed = false;
      for (const o of ws) {
        if (o !== w && o.startsWith(w)) { absorbed = true; break; }
      }
      if (!absorbed) ans += 1;
    }
    return ans;
  },
};
