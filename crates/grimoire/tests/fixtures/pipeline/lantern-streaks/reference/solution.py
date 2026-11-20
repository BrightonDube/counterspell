def longest_lit(lanterns):
    count = 0
    for value in reversed(lanterns):
        if value != 1:
            break
        count += 1
    return count
