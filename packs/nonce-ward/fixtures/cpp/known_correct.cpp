// Test-only known-correct C++ fixture for nonce-ward.
long long nonce_ward(std::vector<std::vector<long long>> msgs) {
    std::map<long long, long long> high;
    long long replays = 0;
    for (auto& m : msgs) {
        auto it = high.find(m[0]);
        long long mark = it == high.end() ? -1 : it->second;
        if (m[1] <= mark) replays++; else high[m[0]] = m[1];
    }
    return replays;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(nonce_ward(from_json<std::vector<std::vector<long long>>>(args[0])));
}
