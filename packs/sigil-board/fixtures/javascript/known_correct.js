module.exports = {
  sigil_count: (n) => {
    const f = (r, cols) => {
      if (r === n) return 1;
      let total = 0;
      for (let c = 0; c < n; c += 1) {
        if (cols.includes(c)) continue;
        let clash = false;
        for (let rr = 0; rr < cols.length; rr += 1) {
          if (Math.abs(r - rr) === Math.abs(c - cols[rr])) {
            clash = true;
            break;
          }
        }
        if (!clash) total += f(r + 1, cols.concat([c]));
      }
      return total;
    };
    return f(0, []);
  },
};
