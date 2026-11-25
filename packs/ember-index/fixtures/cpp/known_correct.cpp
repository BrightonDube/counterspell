// Test-only known-correct C++ fixture for ember-index.
long long find_ember(std::vector<long long> embers, long long target) {
    long long lo = 0, hi = (long long) embers.size();
    while (lo < hi) {
        long long mid = (lo + hi) / 2;
        if (embers[(size_t) mid] < target) { lo = mid + 1; } else { hi = mid; }
    }
    return lo < (long long) embers.size() && embers[(size_t) lo] == target ? lo : -1;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(find_ember(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1])));
}
