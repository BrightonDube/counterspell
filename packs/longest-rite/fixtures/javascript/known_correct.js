module.exports = {
  longest_nested: (inscription) => {
    let best = 0;
    const stack = [-1];
    for (let index = 0; index < inscription.length; index += 1) {
      if (inscription[index] === "(") {
        stack.push(index);
      } else {
        stack.pop();
        if (stack.length > 0) {
          best = Math.max(best, index - stack[stack.length - 1]);
        } else {
          stack.push(index);
        }
      }
    }
    return best;
  },
};
