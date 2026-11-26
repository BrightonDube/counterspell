module.exports = {
  prefix_count: (words, prefix) => {
    const entries = words.split(",").filter((w) => w.length > 0);
    return entries.filter((w) => w.startsWith(prefix)).length;
  },
};
