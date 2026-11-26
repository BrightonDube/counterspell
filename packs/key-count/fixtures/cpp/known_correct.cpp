// Test-only known-correct C++ fixture for key-count.
long long key_count(std::vector<std::vector<long long>> rows) {
    std::set<long long> keys;
    for (auto& r : rows) {
        keys.insert(r[0]);
    }
    return (long long) keys.size();
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(key_count(from_json<std::vector<std::vector<long long>>>(args[0])));
}
