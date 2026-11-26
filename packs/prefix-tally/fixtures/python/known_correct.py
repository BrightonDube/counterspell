def prefix_count(words, prefix):
    entries = [w for w in words.split(",") if w]
    return sum(1 for w in entries if w.startswith(prefix))
