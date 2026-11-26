// Test-only known-correct C++ fixture for ember-grid.
long long grid_min(std::vector<std::vector<long long>> grid) {
    size_t h = grid.size();
    size_t w = grid[0].size();
    std::vector<std::vector<long long>> d(h, std::vector<long long>(w, 0));
    const long long INF = std::numeric_limits<long long>::max() / 4;
    for (size_t i = 0; i < h; i += 1) {
        for (size_t j = 0; j < w; j += 1) {
            if (i == 0 && j == 0) {
                d[i][j] = grid[0][0];
            } else {
                long long best = std::min(i > 0 ? d[i - 1][j] : INF,
                                          j > 0 ? d[i][j - 1] : INF);
                d[i][j] = best + grid[i][j];
            }
        }
    }
    return d[h - 1][w - 1];
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(grid_min(from_json<std::vector<std::vector<long long>>>(args[0])));
}
