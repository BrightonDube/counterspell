def page_turn(pages, f):
    folio = []
    faults = 0
    for p in pages:
        if p not in folio:
            faults += 1
            if len(folio) == f:
                folio.pop(0)
            folio.append(p)
    return faults
