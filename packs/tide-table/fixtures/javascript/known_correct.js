module.exports = {
  first_drop: (heights) => {
    for (let index = 1; index < heights.length; index += 1) {
      if (heights[index] < heights[index - 1]) return index;
    }
    return -1;
  },
};
