def top_marks(scores, k):
    return sum(sorted(scores, reverse=True)[:k])
