// Test-only known-correct C++ fixture for twin-tides (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
std::vector<long long> merge_tides(std::vector<long long> first, std::vector<long long> second) {
    std::vector<long long> merged;
    merged.reserve(first.size() + second.size());
    std::size_t a = 0, b = 0;
    while (a < first.size() && b < second.size()) {
        if (first[a] <= second[b]) merged.push_back(first[a++]);
        else merged.push_back(second[b++]);
    }
    merged.insert(merged.end(), first.begin() + a, first.end());
    merged.insert(merged.end(), second.begin() + b, second.end());
    return merged;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(merge_tides(
        from_json<std::vector<long long>>(args[0]),
        from_json<std::vector<long long>>(args[1])));
}
