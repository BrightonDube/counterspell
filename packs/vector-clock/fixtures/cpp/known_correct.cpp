// Test-only known-correct C++ fixture for vector-clock.
long long clock_cmp(std::vector<long long> a, std::vector<long long> b) {
    bool less = false, more = false;
    for (size_t i = 0; i < a.size(); i++) {
        if (a[i] < b[i]) less = true;
        if (a[i] > b[i]) more = true;
    }
    if (less && more) return 2;
    if (less) return -1;
    if (more) return 1;
    return 0;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(clock_cmp(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1])));
}
