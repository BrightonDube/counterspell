def find_in_rotated(stores, target):
    lo, hi = 0, len(stores) - 1
    while lo <= hi:
        mid = (lo + hi) // 2
        if stores[mid] == target:
            return mid
        if stores[lo] <= stores[mid]:
            if stores[lo] <= target < stores[mid]:
                hi = mid - 1
            else:
                lo = mid + 1
        else:
            if stores[mid] < target <= stores[hi]:
                lo = mid + 1
            else:
                hi = mid - 1
    return -1
