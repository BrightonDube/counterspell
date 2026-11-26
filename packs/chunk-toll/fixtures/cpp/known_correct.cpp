// Test-only known-correct C++ fixture for chunk-toll.
long long chunk_toll(std::vector<long long> tokens, long long w, long long base) {
    long long n = (long long)tokens.size();
    long long width = std::max(w, 1LL);
    std::vector<long long> dp(n + 1, LLONG_MAX / 4);
    dp[0] = 0;
    for (long long i = 1; i <= n; i++) {
        for (long long j = std::max(0LL, i - width); j < i; j++) {
            long long hi = tokens[j], lo = tokens[j];
            for (long long k = j; k < i; k++) { hi = std::max(hi, tokens[k]); lo = std::min(lo, tokens[k]); }
            long long cost = dp[j] + base + (hi - lo);
            if (cost < dp[i]) dp[i] = cost;
        }
    }
    return dp[n];
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(chunk_toll(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1]), from_json<long long>(args[2])));
}
