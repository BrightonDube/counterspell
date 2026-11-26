// Test-only known-correct C++ fixture for grille-toll.
long long toll_grid(std::vector<std::vector<long long>> grid) {
    size_t h = grid.size();
    size_t w = grid[0].size();
    const long long BIG = std::numeric_limits<long long>::max() / 4;
    std::vector<std::vector<long long>> d(h, std::vector<long long>(w, BIG));
    for (size_t i = 0; i < h; i += 1) {
        for (size_t j = 0; j < w; j += 1) {
            if (grid[i][j] != 0) {
                continue;
            }
            if (i == 0 && j == 0) {
                d[i][j] = 1;
            } else {
                long long best = std::min(i > 0 ? d[i - 1][j] : BIG,
                                          j > 0 ? d[i][j - 1] : BIG);
                if (best < BIG) {
                    d[i][j] = best + 1;
                }
            }
        }
    }
    return d[h - 1][w - 1] >= BIG ? -1 : d[h - 1][w - 1];
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(toll_grid(from_json<std::vector<std::vector<long long>>>(args[0])));
}
