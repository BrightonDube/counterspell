def merge_glow(amarks, anxt, bmarks, bnxt):
    out = []
    i, j = (0 if anxt else -1), (0 if bnxt else -1)
    while i != -1 and j != -1:
        if amarks[i] <= bmarks[j]:
            out.append(amarks[i]); i = anxt[i]
        else:
            out.append(bmarks[j]); j = bnxt[j]
    while i != -1:
        out.append(amarks[i]); i = anxt[i]
    while j != -1:
        out.append(bmarks[j]); j = bnxt[j]
    return out
