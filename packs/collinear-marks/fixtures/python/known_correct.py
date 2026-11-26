def collinear_marks(points):
    if len(points) < 2 or points[0] == points[1]:
        return -1
    x0, y0 = points[0]
    dx, dy = points[1][0] - x0, points[1][1] - y0
    return sum(1 for x, y in points if (x - x0) * dy == (y - y0) * dx)
