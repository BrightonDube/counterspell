module.exports = {
  corner_turns: (points) => {
    let count = 0;
    for (let i = 1; i < points.length - 1; i += 1) {
      const ux = points[i][0] - points[i - 1][0];
      const uy = points[i][1] - points[i - 1][1];
      const vx = points[i + 1][0] - points[i][0];
      const vy = points[i + 1][1] - points[i][1];
      if (ux * vy - uy * vx > 0) count += 1;
    }
    return count;
  },
};
