module.exports = {
  common_stem: (words) => {
    const entries = words.split(",").filter((w) => w.length > 0);
    if (entries.length === 0) return 0;
    let stem = entries[0];
    for (const w of entries.slice(1)) {
      while (!w.startsWith(stem)) stem = stem.slice(0, -1);
    }
    return stem.length;
  },
};
