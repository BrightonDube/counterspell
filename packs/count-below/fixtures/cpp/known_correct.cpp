// Test-only known-correct C++ fixture for count-below.
long long count_below(std::vector<long long> marks, long long toll) {
    long long lo = 0, hi = (long long) marks.size();
    while (lo < hi) {
        long long mid = (lo + hi) / 2;
        if (marks[(size_t) mid] < toll) { lo = mid + 1; } else { hi = mid; }
    }
    return lo;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(count_below(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1])));
}
