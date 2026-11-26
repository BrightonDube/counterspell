// Test-only known-correct C++ fixture for group-tally.
long long group_tally(std::vector<std::vector<long long>> rows) {
    std::map<long long, long long> tally;
    for (auto& r : rows) {
        tally[r[0]] += r[1];
    }
    long long best = 0;
    for (auto& [k, v] : tally) {
        best = std::max(best, v);
    }
    return best;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(group_tally(from_json<std::vector<std::vector<long long>>>(args[0])));
}
