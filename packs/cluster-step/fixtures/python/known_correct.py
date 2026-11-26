def cluster_step(points, centroids, labels):
    if not centroids:
        return len(points)
    moved = 0
    for i, p in enumerate(points):
        best = min(range(len(centroids)), key=lambda j: (abs(p - centroids[j]), j))
        if i >= len(labels) or labels[i] != best:
            moved += 1
    return moved
