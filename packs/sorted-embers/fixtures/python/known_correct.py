def ordered_squares(readings):
    answer = [0] * len(readings)
    left, right = 0, len(readings) - 1
    write = len(readings) - 1
    while left <= right:
        lo = readings[left] * readings[left]
        hi = readings[right] * readings[right]
        if hi >= lo:
            answer[write] = hi
            right -= 1
        else:
            answer[write] = lo
            left += 1
        write -= 1
    return answer
