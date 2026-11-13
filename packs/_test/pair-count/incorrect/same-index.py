# Deliberately wrong for the ghost engine's dogfooding demo.
# The inner loop starts at i, so an item is paired with itself.
def pair_count(bones, target):
    count = 0
    for i in range(len(bones)):
        for j in range(i, len(bones)):
            if bones[i] + bones[j] == target:
                count += 1
    return count
