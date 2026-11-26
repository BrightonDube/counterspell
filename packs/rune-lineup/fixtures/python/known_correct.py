def lineups(marks):
    def go(prev, left):
        if not left:
            return 1
        return sum(
            go(left[i], left[:i] + left[i + 1:])
            for i in range(len(left))
            if left[i] != prev
        )
    return go(None, list(marks))
