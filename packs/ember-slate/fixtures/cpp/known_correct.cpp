// Test-only known-correct C++ fixture for ember-slate.
long long slate_len(std::vector<long long> marks) {
    if (marks.empty()) {
        return 0;
    }
    std::vector<long long> d(marks.size(), 1);
    long long best = 1;
    for (size_t i = 0; i < marks.size(); i += 1) {
        for (size_t j = 0; j < i; j += 1) {
            if (marks[j] < marks[i]) {
                d[i] = std::max(d[i], d[j] + 1);
            }
        }
        best = std::max(best, d[i]);
    }
    return best;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(slate_len(from_json<std::vector<long long>>(args[0])));
}
