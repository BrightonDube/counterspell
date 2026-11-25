def surviving_text(chant):
    written = []
    for rune in chant:
        if rune == "#":
            if written:
                written.pop()
        else:
            written.append(rune)
    return "".join(written)
