def drift_dark(marks):
    lit = [mark for mark in marks if mark != 0]
    dark = len(marks) - len(lit)
    return lit + [0] * dark
