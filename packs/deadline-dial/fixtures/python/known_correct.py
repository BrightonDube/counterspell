import heapq


def dial_count(slots):
    jobs = sorted(zip(slots[0::2], slots[1::2]))
    held = []
    time = 0
    for d, t in jobs:
        time += t
        heapq.heappush(held, -t)
        if time > d:
            time += heapq.heappop(held)
    return len(held)
