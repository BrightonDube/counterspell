// Test-only known-correct C++ fixture for twin-ledgers (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
long long matching_totals(std::vector<std::vector<long long>> entries_a,
                          std::vector<std::vector<long long>> entries_b) {
    long long a[51] = {};
    long long b[51] = {};
    bool in_a[51] = {};
    bool in_b[51] = {};
    for (const auto& row : entries_a) {
        a[row[0]] += row[1];
        in_a[row[0]] = true;
    }
    for (const auto& row : entries_b) {
        b[row[0]] += row[1];
        in_b[row[0]] = true;
    }
    long long agreed = 0;
    for (int key = 0; key <= 50; key += 1) {
        if (in_a[key] && in_b[key] && a[key] == b[key]) agreed += 1;
    }
    return agreed;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(matching_totals(
        from_json<std::vector<std::vector<long long>>>(args[0]),
        from_json<std::vector<std::vector<long long>>>(args[1])));
}
