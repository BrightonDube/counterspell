// Test-only known-correct C++ fixture for grille-paths.
static long long f(const std::vector<std::vector<long long>>& grid, size_t r, size_t c) {
    if (r >= grid.size() || c >= grid[0].size() || grid[r][c] == 0) {
        return 0;
    }
    if (r == grid.size() - 1 && c == grid[0].size() - 1) {
        return 1;
    }
    return f(grid, r, c + 1) + f(grid, r + 1, c);
}

long long path_count(std::vector<std::vector<long long>> grid) {
    return f(grid, 0, 0);
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(path_count(from_json<std::vector<std::vector<long long>>>(args[0])));
}
