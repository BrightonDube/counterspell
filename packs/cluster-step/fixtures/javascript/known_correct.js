module.exports = {
  cluster_step: (points, centroids, labels) => {
    if (centroids.length === 0) return points.length;
    let moved = 0;
    for (let i = 0; i < points.length; i++) {
      let best = 0;
      for (let j = 1; j < centroids.length; j++) {
        if (Math.abs(points[i] - centroids[j]) < Math.abs(points[i] - centroids[best])) best = j;
      }
      if (i >= labels.length || labels[i] !== best) moved++;
    }
    return moved;
  },
};
