// Test-only known-correct C++ fixture for scrawl-edit.
long long edit_toll(std::string first, std::string second) {
    size_t n = first.size();
    size_t m = second.size();
    std::vector<std::vector<long long>> d(n + 1, std::vector<long long>(m + 1, 0));
    for (size_t i = 0; i <= n; i += 1) d[i][0] = i;
    for (size_t j = 0; j <= m; j += 1) d[0][j] = j;
    for (size_t i = 1; i <= n; i += 1) {
        for (size_t j = 1; j <= m; j += 1) {
            long long cost = first[i - 1] == second[j - 1] ? 0 : 1;
            d[i][j] = std::min({d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + cost});
        }
    }
    return d[n][m];
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(edit_toll(from_json<std::string>(args[0]), from_json<std::string>(args[1])));
}
