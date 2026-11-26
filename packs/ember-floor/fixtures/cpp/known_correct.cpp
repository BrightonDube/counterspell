// Test-only known-correct C++ fixture for ember-floor.
long long floor_sum(std::vector<long long> marks, long long k) {
    std::sort(marks.begin(), marks.end());
    long long total = 0;
    for (size_t i = 0; i < marks.size() && (long long) i < k; i += 1) {
        total += marks[i];
    }
    return total;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(floor_sum(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
