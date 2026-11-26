// Test-only known-correct C++ fixture for lantern-watch.
long long watch_points(std::vector<std::vector<long long>> spans) {
    spans.erase(std::remove_if(spans.begin(), spans.end(),
                               [](auto& s) { return s[0] > s[1]; }),
                spans.end());
    std::sort(spans.begin(), spans.end(), [](auto& x, auto& y) {
        return x[1] != y[1] ? x[1] < y[1] : x[0] < y[0];
    });
    long long pts = 0;
    size_t i = 0;
    while (i < spans.size()) {
        long long p = spans[i][1];
        pts += 1;
        while (i < spans.size() && spans[i][0] <= p) {
            i += 1;
        }
    }
    return pts;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(watch_points(from_json<std::vector<std::vector<long long>>>(args[0])));
}
