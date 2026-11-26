// Test-only known-correct C++ fixture for depend-rite.
long long depend_rite(std::vector<std::vector<long long>> rows) {
    std::map<long long, long long> seen;
    for (auto& r : rows) {
        if (seen.count(r[0]) && seen[r[0]] != r[1]) {
            return 0;
        }
        seen[r[0]] = r[1];
    }
    return 1;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(depend_rite(from_json<std::vector<std::vector<long long>>>(args[0])));
}
