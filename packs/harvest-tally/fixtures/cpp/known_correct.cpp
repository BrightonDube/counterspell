// Test-only known-correct C++ fixture for harvest-tally (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
// The tally outgrows int, so every accumulator is a long long.
long long cross_pairs(std::vector<long long> rows) {
    long long total = 0;
    long long ahead = 0;
    for (long long count : rows) ahead += count;
    for (long long count : rows) {
        ahead -= count;
        total += count * ahead;
    }
    return total;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(cross_pairs(from_json<std::vector<long long>>(args[0])));
}
