def trapped_water(heights):
    left, right = 0, len(heights) - 1
    left_bound, right_bound = 0, 0
    water = 0
    while left < right:
        if heights[left] <= heights[right]:
            left_bound = max(left_bound, heights[left])
            water += left_bound - heights[left]
            left += 1
        else:
            right_bound = max(right_bound, heights[right])
            water += right_bound - heights[right]
            right -= 1
    return water
