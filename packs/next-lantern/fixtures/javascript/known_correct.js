module.exports = {
  next_taller: (heights) => {
    const answer = new Array(heights.length).fill(0);
    const waiting = [];
    for (let index = heights.length - 1; index >= 0; index -= 1) {
      while (waiting.length > 0 && heights[waiting[waiting.length - 1]] <= heights[index]) {
        waiting.pop();
      }
      if (waiting.length > 0) {
        answer[index] = waiting[waiting.length - 1] - index;
      }
      waiting.push(index);
    }
    return answer;
  },
};
