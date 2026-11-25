def triads_at_quota(marks, quota):
    count = 0
    n = len(marks)
    for i in range(n - 2):
        target = quota - marks[i]
        left, right = i + 1, n - 1
        while left < right:
            pair = marks[left] + marks[right]
            if pair < target:
                left += 1
            elif pair > target:
                right -= 1
            elif marks[left] == marks[right]:
                span = right - left + 1
                count += span * (span - 1) // 2
                break
            else:
                lo = 1
                while left + lo < right and marks[left + lo] == marks[left]:
                    lo += 1
                hi = 1
                while right - hi > left and marks[right - hi] == marks[right]:
                    hi += 1
                count += lo * hi
                left += lo
                right -= hi
    return count
