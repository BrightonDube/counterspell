// Test-only known-correct C++ fixture for valley-low.
long long lowest_mark(std::vector<long long> rotated) {
    long long lo = 0, hi = (long long) rotated.size() - 1;
    while (lo < hi) {
        long long mid = (lo + hi) / 2;
        if (rotated[(size_t) mid] > rotated[(size_t) hi]) { lo = mid + 1; } else { hi = mid; }
    }
    return rotated[(size_t) lo];
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(lowest_mark(from_json<std::vector<long long>>(args[0])));
}
