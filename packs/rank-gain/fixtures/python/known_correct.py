def rank_gain(rels, k):
    return sum(r * (k - i) for i, r in enumerate(rels[:k]))
