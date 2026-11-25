module.exports = {
  widest_fit: (wall) => {
    let best = 0;
    const stack = [];
    for (let index = 0; index <= wall.length; index += 1) {
      const height = index < wall.length ? wall[index] : 0;
      while (stack.length > 0 && height < wall[stack[stack.length - 1]]) {
        const popped = stack.pop();
        const left = stack.length > 0 ? stack[stack.length - 1] + 1 : 0;
        const area = wall[popped] * (index - left);
        if (area > best) {
          best = area;
        }
      }
      stack.push(index);
    }
    return best;
  },
};
