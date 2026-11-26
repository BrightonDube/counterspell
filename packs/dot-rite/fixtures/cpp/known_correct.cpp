// Test-only known-correct C++ fixture for dot-rite.
long long dot_sparse(std::vector<std::vector<long long>> a, std::vector<std::vector<long long>> b) {
    std::map<long long, long long> da, db;
    for (auto& r : a) da[r[0]] = r[1];
    for (auto& r : b) db[r[0]] = r[1];
    long long total = 0;
    for (auto& [p, m] : da) if (db.count(p)) total += m * db[p];
    return total;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(dot_sparse(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<std::vector<std::vector<long long>>>(args[1])));
}
