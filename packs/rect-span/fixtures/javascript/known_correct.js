module.exports = {
  rect_span: (a, b) => {
    const [ax1, ax2] = [Math.min(a[0], a[2]), Math.max(a[0], a[2])];
    const [ay1, ay2] = [Math.min(a[1], a[3]), Math.max(a[1], a[3])];
    const [bx1, bx2] = [Math.min(b[0], b[2]), Math.max(b[0], b[2])];
    const [by1, by2] = [Math.min(b[1], b[3]), Math.max(b[1], b[3])];
    const w = Math.min(ax2, bx2) - Math.max(ax1, bx1);
    const h = Math.min(ay2, by2) - Math.max(ay1, by1);
    return Math.max(w, 0) * Math.max(h, 0);
  },
};
