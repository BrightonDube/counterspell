def longest_nested(inscription):
    best = 0
    stack = [-1]
    for index, rune in enumerate(inscription):
        if rune == "(":
            stack.append(index)
        else:
            stack.pop()
            if stack:
                best = max(best, index - stack[-1])
            else:
                stack.append(index)
    return best
