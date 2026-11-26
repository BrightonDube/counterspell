// Test-only known-correct C++ fixture for rush-slots.
long long pick_count(std::vector<std::vector<long long>> spans) {
    std::sort(spans.begin(), spans.end(), [](auto& x, auto& y) {
        return x[1] != y[1] ? x[1] < y[1] : x[0] < y[0];
    });
    long long n = 0;
    long long end = -1;
    for (auto& s : spans) {
        if (s[0] > end) {
            n += 1;
            end = s[1];
        }
    }
    return n;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(pick_count(from_json<std::vector<std::vector<long long>>>(args[0])));
}
