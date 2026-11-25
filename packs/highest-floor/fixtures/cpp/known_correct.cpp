// Test-only known-correct C++ fixture for highest-floor.
long long highest_floor(std::vector<long long> steps, long long reach) {
    long long lo = 0, hi = (long long) steps.size();
    while (lo < hi) {
        long long mid = (lo + hi) / 2;
        if (steps[(size_t) mid] <= reach) { lo = mid + 1; } else { hi = mid; }
    }
    return lo > 0 ? steps[(size_t) lo - 1] : -1;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(highest_floor(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1])));
}
