// Test-only known-correct C++ fixture for twin-echoes (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
long long repeating_echoes(std::vector<long long> echoes) {
    long long counts[51] = {};
    for (long long echo : echoes) counts[echo] += 1;
    long long total = 0;
    for (long long count : counts) {
        if (count >= 2) total += 1;
    }
    return total;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(repeating_echoes(from_json<std::vector<long long>>(args[0])));
}
