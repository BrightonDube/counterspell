def quorum_reached(n, votes):
    return 1 if votes >= n // 2 + 1 else 0
