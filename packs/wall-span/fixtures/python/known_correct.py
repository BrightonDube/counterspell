def widest_fit(wall):
    best = 0
    stack = []
    for index in range(len(wall) + 1):
        height = wall[index] if index < len(wall) else 0
        while stack and height < wall[stack[-1]]:
            popped = stack.pop()
            left = stack[-1] + 1 if stack else 0
            best = max(best, wall[popped] * (index - left))
        stack.append(index)
    return best
