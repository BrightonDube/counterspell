module.exports = {
  ordered_squares: (readings) => {
    const answer = new Array(readings.length);
    let left = 0;
    let right = readings.length - 1;
    let write = right;
    while (left <= right) {
      const lo = readings[left] * readings[left];
      const hi = readings[right] * readings[right];
      if (hi >= lo) {
        answer[write] = hi;
        right -= 1;
      } else {
        answer[write] = lo;
        left += 1;
      }
      write -= 1;
    }
    return answer;
  },
};
