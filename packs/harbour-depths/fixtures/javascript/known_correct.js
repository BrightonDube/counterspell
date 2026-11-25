module.exports = {
  deepest_water: (heights) => {
    let left = 0;
    let right = heights.length - 1;
    let deepest = 0;
    while (left < right) {
      const held = Math.min(heights[left], heights[right]) * (right - left);
      if (held > deepest) deepest = held;
      if (heights[left] <= heights[right]) left += 1;
      else right -= 1;
    }
    return deepest;
  },
};
