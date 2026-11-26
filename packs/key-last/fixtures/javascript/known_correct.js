module.exports = {
  key_last: (rows, k) => {
    let hit = -1;
    for (const [key, v] of rows) {
      if (key === k) hit = v;
    }
    return hit;
  },
};
