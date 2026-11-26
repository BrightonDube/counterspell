// Test-only known-correct C++ fixture for ember-echo.
long long echo_len(std::string first, std::string second) {
    size_t n = first.size();
    size_t m = second.size();
    std::vector<std::vector<long long>> d(n + 1, std::vector<long long>(m + 1, 0));
    for (size_t i = 1; i <= n; i += 1) {
        for (size_t j = 1; j <= m; j += 1) {
            if (first[i - 1] == second[j - 1]) {
                d[i][j] = d[i - 1][j - 1] + 1;
            } else {
                d[i][j] = std::max(d[i - 1][j], d[i][j - 1]);
            }
        }
    }
    return d[n][m];
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(echo_len(from_json<std::string>(args[0]), from_json<std::string>(args[1])));
}
