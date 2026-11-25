module.exports = {
  repeating_echoes: (echoes) => {
    const counts = new Map();
    for (const echo of echoes) counts.set(echo, (counts.get(echo) ?? 0) + 1);
    let total = 0;
    for (const count of counts.values()) {
      if (count >= 2) total += 1;
    }
    return total;
  },
};
