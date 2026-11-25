// Test-only known-correct C++ fixture for shared-sigils (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
std::vector<long long> shared_marks(std::vector<long long> first, std::vector<long long> second) {
    std::vector<long long> shared;
    std::size_t a = 0, b = 0;
    while (a < first.size() && b < second.size()) {
        if (first[a] < second[b]) {
            a += 1;
        } else if (second[b] < first[a]) {
            b += 1;
        } else {
            long long mark = first[a];
            if (shared.empty() || shared.back() != mark) {
                shared.push_back(mark);
            }
            a += 1;
            b += 1;
        }
    }
    return shared;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(shared_marks(
        from_json<std::vector<long long>>(args[0]),
        from_json<std::vector<long long>>(args[1])));
}
