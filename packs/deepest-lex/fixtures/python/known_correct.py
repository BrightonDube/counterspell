def longest_word(words):
    return max((len(w) for w in words.split(",") if w), default=0)
