def deepest_water(heights):
    left, right = 0, len(heights) - 1
    deepest = 0
    while left < right:
        held = min(heights[left], heights[right]) * (right - left)
        deepest = max(deepest, held)
        if heights[left] <= heights[right]:
            left += 1
        else:
            right -= 1
    return deepest
