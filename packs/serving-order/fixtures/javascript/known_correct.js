module.exports = {
  serve_order: (ops) => {
    const line = [];
    const served = [];
    for (const op of ops) {
      if (op === 0) {
        if (line.length > 0) {
          served.push(line.shift());
        }
      } else {
        line.push(op);
      }
    }
    return served;
  },
};
