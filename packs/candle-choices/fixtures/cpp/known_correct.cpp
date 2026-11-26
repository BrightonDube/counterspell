// Test-only known-correct C++ fixture for candle-choices.
long long pick_ways(long long n, long long k) {
    if (k < 0 || k > n) {
        return 0;
    }
    if (k == 0) {
        return 1;
    }
    return pick_ways(n - 1, k - 1) + pick_ways(n - 1, k);
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(pick_ways(from_json<long long>(args[0]), from_json<long long>(args[1])));
}
