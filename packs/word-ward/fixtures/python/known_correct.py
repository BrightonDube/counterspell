def has_word(words, word):
    if word == "":
        return 0
    return 1 if word in [w for w in words.split(",") if w] else 0
