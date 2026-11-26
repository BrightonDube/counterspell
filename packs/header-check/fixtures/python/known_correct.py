def header_check(env):
    v, s, q, l = env
    ok = v in (1, 2) and 0 <= s <= 255 and 0 <= q <= 65535 and 0 <= l <= 1500
    return 1 if ok else 0
