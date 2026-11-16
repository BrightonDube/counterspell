def longest_run(text):
    best = 0
    streak = 0
    last = None
    for char in text:
        streak = streak + 1 if char == last else 1
        last = char
        best = max(best, streak)
    return best
