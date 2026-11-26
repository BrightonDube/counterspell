// Test-only known-correct C++ fixture for coin-rite.
long long coin_rite(std::vector<long long> coins, long long target) {
    const long long INF = std::numeric_limits<long long>::max() / 4;
    std::vector<long long> d(target + 1, INF);
    d[0] = 0;
    for (long long i = 1; i <= target; i += 1) {
        for (long long c : coins) {
            if (i - c >= 0) {
                d[i] = std::min(d[i], d[i - c] + 1);
            }
        }
    }
    return d[target] >= INF ? -1 : d[target];
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(coin_rite(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
