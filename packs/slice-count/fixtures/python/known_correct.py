def slice_count(bursts, q):
    return sum((b + q - 1) // q for b in bursts)
