// Test-only known-correct C++ fixture for retry-dedup.
long long retry_dedup(std::vector<std::vector<long long>> log) {
    std::set<long long> seen;
    for (auto& row : log) seen.insert(row[1]);
    return (long long)seen.size();
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(retry_dedup(from_json<std::vector<std::vector<long long>>>(args[0])));
}
