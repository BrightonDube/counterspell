// Test-only known-correct C++ fixture for ember-braid.
long long braid_ways(std::string text, std::string first, std::string second) {
    size_t n = first.size();
    size_t m = second.size();
    if (text.size() != n + m) {
        return 0;
    }
    std::vector<std::vector<long long>> d(n + 1, std::vector<long long>(m + 1, 0));
    d[0][0] = 1;
    for (size_t i = 0; i <= n; i += 1) {
        for (size_t j = 0; j <= m; j += 1) {
            if (i > 0 && first[i - 1] == text[i + j - 1]) {
                d[i][j] |= d[i - 1][j];
            }
            if (j > 0 && second[j - 1] == text[i + j - 1]) {
                d[i][j] |= d[i][j - 1];
            }
        }
    }
    return d[n][m];
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(braid_ways(from_json<std::string>(args[0]), from_json<std::string>(args[1]), from_json<std::string>(args[2])));
}
