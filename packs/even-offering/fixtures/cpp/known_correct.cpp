// Test-only known-correct C++ fixture for even-offering.
static bool ok(const std::vector<long long>& offers, long long cap, long long parts) {
    long long need = 1, cur = 0;
    for (long long v : offers) {
        if (cur + v > cap) { need += 1; cur = 0; }
        cur += v;
    }
    return need <= parts;
}

long long smallest_largest(std::vector<long long> offers, long long parts) {
    long long lo = 0, hi = 0;
    for (long long v : offers) { if (v > lo) { lo = v; } hi += v; }
    while (lo < hi) {
        long long mid = (lo + hi) / 2;
        if (ok(offers, mid, parts)) { hi = mid; } else { lo = mid + 1; }
    }
    return lo;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(smallest_largest(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1])));
}
