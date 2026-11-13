# Deliberately wrong. The shortcut is valid for non-empty lists, but min() raises
# ValueError on an empty list.
def pair_count(bones, target):
    if target < 2 * min(bones):
        return 0
    count = 0
    for i in range(len(bones)):
        for j in range(i + 1, len(bones)):
            if bones[i] + bones[j] == target:
                count += 1
    return count
