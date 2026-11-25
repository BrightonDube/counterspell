module.exports = {
  servings: (ops) => {
    const line = [];
    let served = 0;
    for (const op of ops) {
      if (op === 0) {
        if (line.length > 0) {
          line.shift();
          served += 1;
        }
      } else {
        line.push(op);
      }
    }
    return served;
  },
};
