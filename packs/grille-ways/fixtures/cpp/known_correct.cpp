// Test-only known-correct C++ fixture for grille-ways.
long long grille_ways(long long rows, long long cols) {
    if (rows <= 0 || cols <= 0) {
        return 0;
    }
    std::vector<std::vector<long long>> d(rows, std::vector<long long>(cols, 0));
    for (long long i = 0; i < rows; i += 1) {
        for (long long j = 0; j < cols; j += 1) {
            d[i][j] = (i == 0 || j == 0) ? 1 : d[i - 1][j] + d[i][j - 1];
        }
    }
    return d[rows - 1][cols - 1];
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(grille_ways(from_json<long long>(args[0]), from_json<long long>(args[1])));
}
