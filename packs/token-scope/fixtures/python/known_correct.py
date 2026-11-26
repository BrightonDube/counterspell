def scope_ok(required, granted, expiry, now):
    alive = now < expiry
    covered = all(s in granted for s in required)
    return 1 if alive and covered else 0
