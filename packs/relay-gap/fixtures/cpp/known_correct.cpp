// Test-only known-correct C++ fixture for relay-gap.
long long relay_count(std::vector<std::vector<long long>> spans, long long gap) {
    std::sort(spans.begin(), spans.end(), [](auto& x, auto& y) {
        return x[1] != y[1] ? x[1] < y[1] : x[0] < y[0];
    });
    long long n = 0;
    long long end = -1000000000LL;
    for (auto& s : spans) {
        if (s[0] >= end + gap) {
            n += 1;
            end = s[1];
        }
    }
    return n;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(relay_count(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<long long>(args[1])));
}
