def drain_order(events):
    pairs = list(zip(events[0::2], events[1::2]))
    return [i for _, i in sorted(pairs)]
