def rect_span(a, b):
    ax1, ax2 = sorted((a[0], a[2]))
    ay1, ay2 = sorted((a[1], a[3]))
    bx1, bx2 = sorted((b[0], b[2]))
    by1, by2 = sorted((b[1], b[3]))
    w = min(ax2, bx2) - max(ax1, bx1)
    h = min(ay2, by2) - max(ay1, by1)
    return max(w, 0) * max(h, 0)
