// Test-only known-correct C++ fixture for cover-span.
long long span_cover(std::vector<std::vector<long long>> spans, long long t) {
    std::sort(spans.begin(), spans.end());
    long long front = 0;
    size_t i = 0;
    long long n = 0;
    while (front <= t) {
        long long best = -1;
        while (i < spans.size() && spans[i][0] <= front) {
            best = std::max(best, spans[i][1]);
            i += 1;
        }
        if (best < front) {
            return -1;
        }
        n += 1;
        if (best >= t) {
            return n;
        }
        front = best + 1;
    }
    return n;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(span_cover(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<long long>(args[1])));
}
