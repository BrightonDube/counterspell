import heapq


def meld_cost(sizes):
    heap = list(sizes)
    heapq.heapify(heap)
    cost = 0
    while len(heap) > 1:
        a = heapq.heappop(heap)
        b = heapq.heappop(heap)
        cost += a + b
        heapq.heappush(heap, a + b)
    return cost
