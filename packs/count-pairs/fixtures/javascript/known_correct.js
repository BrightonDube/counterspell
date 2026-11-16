module.exports = {
  count_pairs: (values, target) => {
    let pairs = 0;
    for (let left = 0; left < values.length; left += 1) {
      for (let right = left + 1; right < values.length; right += 1) {
        if (values[left] + values[right] === target) {
          pairs += 1;
        }
      }
    }
    return pairs;
  },
};
