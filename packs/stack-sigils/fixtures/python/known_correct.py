def top_sigil(ops):
    marks = []
    for op in ops:
        if op == 0:
            if marks:
                marks.pop()
        else:
            marks.append(op)
    return marks[-1] if marks else -1
