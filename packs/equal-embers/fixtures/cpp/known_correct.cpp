// Test-only known-correct C++ fixture for equal-embers (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
long long balanced_span(std::vector<long long> embers) {
    std::unordered_map<long long, long long> first_seen{{0, -1}};
    long long balance = 0;
    long long longest = 0;
    for (std::size_t index = 0; index < embers.size(); index += 1) {
        balance += embers[index] == 1 ? 1 : -1;
        auto found = first_seen.find(balance);
        if (found != first_seen.end()) {
            longest = std::max(longest, static_cast<long long>(index) - found->second);
        } else {
            first_seen[balance] = index;
        }
    }
    return longest;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(balanced_span(from_json<std::vector<long long>>(args[0])));
}
