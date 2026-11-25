// Test-only known-correct C++ fixture for sigil-dedup (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
long long distinct_sigils(std::vector<long long> marks) {
    long long distinct = 0;
    for (std::size_t index = 0; index < marks.size(); index += 1) {
        if (index == 0 || marks[index] != marks[index - 1]) {
            distinct += 1;
        }
    }
    return distinct;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(distinct_sigils(from_json<std::vector<long long>>(args[0])));
}
