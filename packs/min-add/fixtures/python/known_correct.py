def fewest_inserts(inscription):
    open_layers = 0
    stranded = 0
    for rune in inscription:
        if rune == "(":
            open_layers += 1
        elif open_layers > 0:
            open_layers -= 1
        else:
            stranded += 1
    return open_layers + stranded
