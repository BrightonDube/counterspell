module.exports = {
  longest_bead: (strand) => {
    let best = 0;
    let run = 0;
    let previous = "";
    for (const bead of strand) {
      run = bead === previous ? run + 1 : 1;
      if (run > best) {
        best = run;
      }
      previous = bead;
    }
    return best;
  },
};
