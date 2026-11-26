// Test-only known-correct C++ fixture for stale-tally.
long long stale_count(std::vector<long long> seen, std::vector<long long> queued) {
    long long n = 0;
    for (size_t i = 0; i < seen.size(); i += 1) {
        if (seen[i] != queued[i]) {
            n += 1;
        }
    }
    return n;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(stale_count(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1])));
}
