// Test-only known-correct C++ fixture for rune-toll.
long long toll_sum(std::vector<std::vector<long long>> edges, std::vector<long long> weights, long long n) {
    long long total = 0;
    size_t len = std::min(edges.size(), weights.size());
    for (size_t k = 0; k < len; k += 1) {
        total += weights[k];
    }
    return total;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(toll_sum(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<long long>(args[2])));
}
