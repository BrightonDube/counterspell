def lru_rite(pages, f):
    folio = []
    faults = 0
    for p in pages:
        if p in folio:
            folio.remove(p)
            folio.append(p)
        else:
            faults += 1
            if len(folio) == f:
                folio.pop(0)
            folio.append(p)
    return faults
