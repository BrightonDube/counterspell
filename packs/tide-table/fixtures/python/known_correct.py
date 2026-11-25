def first_drop(heights):
    for index in range(1, len(heights)):
        if heights[index] < heights[index - 1]:
            return index
    return -1
