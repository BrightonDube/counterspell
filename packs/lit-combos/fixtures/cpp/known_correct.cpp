// Test-only known-correct C++ fixture for lit-combos.
long long count_mixes(std::vector<long long> coins, long long target) {
    std::vector<long long> d(target + 1, 0);
    d[0] = 1;
    for (long long c : coins) {
        for (long long i = c; i <= target; i += 1) {
            d[i] += d[i - c];
        }
    }
    return d[target];
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(count_mixes(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
