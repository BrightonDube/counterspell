// Test-only known-correct C++ fixture for ember-edges.
static long long lower(const std::vector<long long>& embers, long long mark) {
    long long lo = 0, hi = (long long) embers.size();
    while (lo < hi) {
        long long mid = (lo + hi) / 2;
        if (embers[(size_t) mid] < mark) { lo = mid + 1; } else { hi = mid; }
    }
    return lo;
}

std::vector<long long> ember_edges(std::vector<long long> embers, long long target) {
    long long first = lower(embers, target);
    if (first == (long long) embers.size() || embers[(size_t) first] != target) {
        return {-1, -1};
    }
    return {first, lower(embers, target + 1) - 1};
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(ember_edges(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1])));
}
