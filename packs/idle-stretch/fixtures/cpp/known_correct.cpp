// Test-only known-correct C++ fixture for idle-stretch.
long long idle_stretch(std::vector<std::vector<long long>> spans, long long t) {
    std::vector<bool> cov(t + 1, false);
    for (auto& s : spans) {
        for (long long i = std::max(s[0], 0LL); i <= std::min(s[1], t); i += 1) {
            cov[i] = true;
        }
    }
    long long best = 0;
    long long run = 0;
    for (long long i = 0; i <= t; i += 1) {
        run = cov[i] ? 0 : run + 1;
        best = std::max(best, run);
    }
    return best;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(idle_stretch(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<long long>(args[1])));
}
