// Test-only known-correct C++ fixture for diagonal-total.
long long diagonal_total(std::vector<std::vector<long long>> grid) {
    long long edge = grid.empty() ? 0 : std::min<long long>(grid.size(), (long long)grid[0].size());
    long long total = 0;
    for (long long row = 0; row < edge; row++) {
        total += grid[row][row];
    }
    return total;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(diagonal_total(from_json<std::vector<std::vector<long long>>>(args[0])));
}
