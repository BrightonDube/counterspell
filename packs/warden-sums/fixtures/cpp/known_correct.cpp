// Test-only known-correct C++ fixture for warden-sums (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
long long ritual_spans(std::vector<long long> readings, long long quota) {
    std::unordered_map<long long, long long> seen{{0, 1}};
    long long total = 0;
    long long spans = 0;
    for (long long reading : readings) {
        total += reading;
        auto found = seen.find(total - quota);
        if (found != seen.end()) spans += found->second;
        seen[total] += 1;
    }
    return spans;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(ritual_spans(from_json<std::vector<long long>>(args[0]),
                                from_json<long long>(args[1])));
}
