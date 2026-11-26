// Test-only known-correct C++ fixture for toll-stairs.
long long toll_stairs(std::vector<long long> costs) {
    long long n = (long long) costs.size();
    if (n <= 1) {
        return n == 1 ? costs[0] : 0;
    }
    std::vector<long long> d(n + 1, 0);
    for (long long i = 2; i <= n; i += 1) {
        d[i] = std::min(d[i - 1] + costs[i - 1], d[i - 2] + costs[i - 2]);
    }
    return d[n];
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(toll_stairs(from_json<std::vector<long long>>(args[0])));
}
