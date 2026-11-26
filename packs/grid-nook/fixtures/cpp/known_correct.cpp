// Test-only known-correct C++ fixture for grid-nook.
long long nook_size(std::vector<std::vector<long long>> grid) {
    size_t rows = grid.size();
    size_t cols = grid[0].size();
    if (grid[0][0] == 0) {
        return 0;
    }
    std::vector<std::vector<char>> seen(rows, std::vector<char>(cols, 0));
    seen[0][0] = 1;
    std::vector<std::pair<size_t, size_t>> stack{{0, 0}};
    const int dirs[4][2] = {{0, 1}, {0, -1}, {1, 0}, {-1, 0}};
    while (!stack.empty()) {
        auto cell = stack.back();
        stack.pop_back();
        for (auto& d : dirs) {
            long long nr = (long long) cell.first + d[0];
            long long nc = (long long) cell.second + d[1];
            if (nr >= 0 && nr < (long long) rows && nc >= 0 && nc < (long long) cols
                && grid[nr][nc] == 1 && !seen[nr][nc]) {
                seen[nr][nc] = 1;
                stack.emplace_back((size_t) nr, (size_t) nc);
            }
        }
    }
    long long count = 0;
    for (auto& row : seen) {
        for (char v : row) {
            if (v) {
                count += 1;
            }
        }
    }
    return count;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(nook_size(from_json<std::vector<std::vector<long long>>>(args[0])));
}
