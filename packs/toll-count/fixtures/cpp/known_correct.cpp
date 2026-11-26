// Test-only known-correct C++ fixture for toll-count.
long long toll_count(std::vector<std::vector<long long>> edges, std::vector<long long> weights, long long t) {
    long long count = 0;
    size_t len = std::min(edges.size(), weights.size());
    for (size_t k = 0; k < len; k += 1) {
        if (weights[k] <= t) {
            count += 1;
        }
    }
    return count;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(toll_count(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<long long>(args[2])));
}
