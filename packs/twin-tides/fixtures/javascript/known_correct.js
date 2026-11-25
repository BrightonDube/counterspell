module.exports = {
  merge_tides: (first, second) => {
    const merged = [];
    let a = 0;
    let b = 0;
    while (a < first.length && b < second.length) {
      if (first[a] <= second[b]) {
        merged.push(first[a]);
        a += 1;
      } else {
        merged.push(second[b]);
        b += 1;
      }
    }
    return merged.concat(first.slice(a), second.slice(b));
  },
};
