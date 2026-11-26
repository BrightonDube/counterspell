def corner_turns(points):
    count = 0
    for i in range(1, len(points) - 1):
        ux = points[i][0] - points[i - 1][0]
        uy = points[i][1] - points[i - 1][1]
        vx = points[i + 1][0] - points[i][0]
        vy = points[i + 1][1] - points[i][1]
        if ux * vy - uy * vx > 0:
            count += 1
    return count
