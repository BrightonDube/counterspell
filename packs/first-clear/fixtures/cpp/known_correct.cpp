// Test-only known-correct C++ fixture for first-clear.
long long first_clear(std::vector<long long> heights, long long mark) {
    long long lo = 0, hi = (long long) heights.size();
    while (lo < hi) {
        long long mid = (lo + hi) / 2;
        if (heights[(size_t) mid] < mark) { lo = mid + 1; } else { hi = mid; }
    }
    return lo;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(first_clear(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1])));
}
