def lantern_ledger(changes, start, level):
    meter = start
    marks = 0
    for change in changes:
        meter += change
        if meter == level:
            marks += 1
    return marks
