def null_marks(rows):
    return sum(1 for _, v in rows if v == -1)
