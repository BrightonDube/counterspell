def pair_count(bones, target):
    count = 0
    for i in range(len(bones)):
        for j in range(i + 1, len(bones)):
            if bones[i] + bones[j] == target:
                count += 1
    return count
