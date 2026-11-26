module.exports = {
  can_segment: (words, s) => {
    const ws = new Set(words.split(",").filter((w) => w.length > 0));
    const ok = new Array(s.length + 1).fill(false);
    ok[0] = true;
    for (let i = 0; i <= s.length; i += 1) {
      if (!ok[i]) continue;
      for (const w of ws) {
        if (s.startsWith(w, i)) ok[i + w.length] = true;
      }
    }
    return ok[s.length] ? 1 : 0;
  },
};
