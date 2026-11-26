// Test-only known-correct C++ fixture for join-span.
long long join_span(std::vector<std::vector<long long>> a,
                    std::vector<std::vector<long long>> b) {
    long long count = 0;
    for (auto& x : a) {
        for (auto& y : b) {
            if (x[0] == y[0] && x[1] < y[1]) {
                count += 1;
            }
        }
    }
    return count;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(join_span(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<std::vector<std::vector<long long>>>(args[1])));
}
