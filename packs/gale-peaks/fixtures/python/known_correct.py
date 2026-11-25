def gale_peaks(readings, width):
    from collections import deque
    peaks = []
    candidates = deque()
    for i, reading in enumerate(readings):
        while candidates and readings[candidates[-1]] <= reading:
            candidates.pop()
        candidates.append(i)
        if candidates[0] <= i - width:
            candidates.popleft()
        if i >= width - 1:
            peaks.append(readings[candidates[0]])
    return peaks
