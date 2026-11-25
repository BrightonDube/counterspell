module.exports = {
  trapped_water: (heights) => {
    let left = 0;
    let right = heights.length - 1;
    let leftBound = 0;
    let rightBound = 0;
    let water = 0;
    while (left < right) {
      if (heights[left] <= heights[right]) {
        leftBound = Math.max(leftBound, heights[left]);
        water += leftBound - heights[left];
        left += 1;
      } else {
        rightBound = Math.max(rightBound, heights[right]);
        water += rightBound - heights[right];
        right -= 1;
      }
    }
    return water;
  },
};
