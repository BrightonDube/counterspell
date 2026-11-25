def repeating_echoes(echoes):
    counts = {}
    for echo in echoes:
        counts[echo] = counts.get(echo, 0) + 1
    return sum(1 for count in counts.values() if count >= 2)
