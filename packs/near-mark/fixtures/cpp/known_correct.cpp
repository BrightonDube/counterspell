// Test-only known-correct C++ fixture for near-mark.
long long near_mark(std::vector<std::vector<long long>> edges, std::vector<long long> weights, long long n) {
    long long best = -1;
    bool found = false;
    size_t len = std::min(edges.size(), weights.size());
    for (size_t k = 0; k < len; k += 1) {
        long long a = edges[k][0];
        long long b = edges[k][1];
        long long c = weights[k];
        if (a == 0 || b == 0) {
            if (!found || c < best) {
                best = c;
                found = true;
            }
        }
    }
    return found ? best : -1;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(near_mark(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<long long>(args[2])));
}
