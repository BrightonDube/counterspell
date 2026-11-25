def remaining_candles(marks, cursed):
    kept = 0
    for mark in marks:
        if mark != cursed:
            kept += 1
    return kept
