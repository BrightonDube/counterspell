// Test-only known-correct C++ fixture for nook-cells.
long long nook_ways(std::vector<std::vector<long long>> grid) {
    size_t h = grid.size();
    size_t w = grid[0].size();
    std::vector<std::vector<long long>> d(h, std::vector<long long>(w, 0));
    for (size_t i = 0; i < h; i += 1) {
        for (size_t j = 0; j < w; j += 1) {
            if (grid[i][j] == 0) {
                continue;
            }
            if (i == 0 && j == 0) {
                d[i][j] = 1;
            } else {
                d[i][j] = (i > 0 ? d[i - 1][j] : 0) + (j > 0 ? d[i][j - 1] : 0);
            }
        }
    }
    return d[h - 1][w - 1];
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(nook_ways(from_json<std::vector<std::vector<long long>>>(args[0])));
}
