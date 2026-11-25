def distinct_sigils(marks):
    distinct = 0
    previous = None
    for mark in marks:
        if mark != previous:
            distinct += 1
            previous = mark
    return distinct
