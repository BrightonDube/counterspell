// Test-only known-correct C++ fixture for nest-count.
long long nest_pairs(std::vector<std::vector<long long>> spans) {
    long long n = 0;
    for (size_t i = 0; i < spans.size(); i += 1) {
        for (size_t j = i + 1; j < spans.size(); j += 1) {
            long long a = spans[i][0], b = spans[i][1];
            long long c = spans[j][0], d = spans[j][1];
            if ((a < c && d < b) || (c < a && b < d)) {
                n += 1;
            }
        }
    }
    return n;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(nest_pairs(from_json<std::vector<std::vector<long long>>>(args[0])));
}
