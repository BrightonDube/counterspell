module.exports = {
  drift_dark: (marks) => {
    const lit = marks.filter((mark) => mark !== 0);
    while (lit.length < marks.length) lit.push(0);
    return lit;
  },
};
