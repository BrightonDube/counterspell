def rate_gate(events, capacity, refill):
    bucket, last, passed = capacity, 0, 0
    for tick, take in events:
        bucket = min(capacity, bucket + (tick - last) * refill)
        last = tick
        if take <= bucket:
            bucket -= take
            passed += 1
    return passed
