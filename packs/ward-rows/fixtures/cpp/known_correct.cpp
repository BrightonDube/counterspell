// Test-only known-correct C++ fixture for ward-rows.
long long row_ways(long long length) {
    if (length <= 0) {
        return 1;
    }
    std::vector<long long> d(length + 1, 0);
    d[0] = 1;
    for (long long i = 1; i <= length; i += 1) {
        for (long long p = 1; p <= 3; p += 1) {
            if (i - p >= 0) {
                d[i] += d[i - p];
            }
        }
    }
    return d[length];
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(row_ways(from_json<long long>(args[0])));
}
