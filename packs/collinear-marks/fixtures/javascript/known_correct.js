module.exports = {
  collinear_marks: (points) => {
    if (points.length < 2 || (points[0][0] === points[1][0] && points[0][1] === points[1][1])) return -1;
    const [x0, y0] = points[0];
    const dx = points[1][0] - x0;
    const dy = points[1][1] - y0;
    return points.filter(([x, y]) => (x - x0) * dy === (y - y0) * dx).length;
  },
};
