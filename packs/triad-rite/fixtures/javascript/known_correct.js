module.exports = {
  triads_at_quota: (marks, quota) => {
    let count = 0;
    const n = marks.length;
    for (let i = 0; i + 2 < n; i += 1) {
      const target = quota - marks[i];
      let left = i + 1;
      let right = n - 1;
      while (left < right) {
        const pair = marks[left] + marks[right];
        if (pair < target) {
          left += 1;
        } else if (pair > target) {
          right -= 1;
        } else if (marks[left] === marks[right]) {
          const span = right - left + 1;
          count += (span * (span - 1)) / 2;
          break;
        } else {
          let lo = 1;
          while (left + lo < right && marks[left + lo] === marks[left]) lo += 1;
          let hi = 1;
          while (right - hi > left && marks[right - hi] === marks[right]) hi += 1;
          count += lo * hi;
          left += lo;
          right -= hi;
        }
      }
    }
    return count;
  },
};
