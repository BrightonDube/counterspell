// Test-only known-correct C++ fixture for ward-spread.
long long spread_ways(long long cells, long long guards) {
    if (guards == 0) {
        return 1;
    }
    if (cells <= 0) {
        return 0;
    }
    return spread_ways(cells - 2, guards - 1) + spread_ways(cells - 1, guards);
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(spread_ways(from_json<long long>(args[0]), from_json<long long>(args[1])));
}
