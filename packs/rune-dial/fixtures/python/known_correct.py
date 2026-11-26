import heapq


def run_median(marks):
    lo = []
    hi = []
    out = []
    for v in marks:
        heapq.heappush(lo, -v)
        heapq.heappush(hi, -heapq.heappop(lo))
        if len(hi) > len(lo):
            heapq.heappush(lo, -heapq.heappop(hi))
        if len(lo) == len(hi):
            out.append(-lo[0] + hi[0])
        else:
            out.append(-2 * lo[0])
    return out
