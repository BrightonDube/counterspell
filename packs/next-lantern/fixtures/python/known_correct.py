def next_taller(heights):
    answer = [0] * len(heights)
    waiting = []
    for index in range(len(heights) - 1, -1, -1):
        while waiting and heights[waiting[-1]] <= heights[index]:
            waiting.pop()
        if waiting:
            answer[index] = waiting[-1] - index
        waiting.append(index)
    return answer
