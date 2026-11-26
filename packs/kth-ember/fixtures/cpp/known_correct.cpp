// Test-only known-correct C++ fixture for kth-ember.
long long kth_mark(std::vector<long long> marks, long long k) {
    std::sort(marks.begin(), marks.end());
    if (k > 0 && (size_t) k <= marks.size()) {
        return marks[(size_t) k - 1];
    }
    return -1;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(kth_mark(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
