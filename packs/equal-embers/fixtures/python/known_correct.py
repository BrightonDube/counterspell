def balanced_span(embers):
    first_seen = {0: -1}
    balance = 0
    longest = 0
    for index, ember in enumerate(embers):
        balance += 1 if ember == 1 else -1
        if balance in first_seen:
            longest = max(longest, index - first_seen[balance])
        else:
            first_seen[balance] = index
    return longest
