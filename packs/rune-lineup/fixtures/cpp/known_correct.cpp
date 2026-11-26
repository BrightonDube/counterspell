// Test-only known-correct C++ fixture for rune-lineup.
static long long go(long long prev, bool has_prev, std::vector<long long>& left) {
    if (left.empty()) {
        return 1;
    }
    long long total = 0;
    for (size_t i = 0; i < left.size(); i += 1) {
        if (!has_prev || left[i] != prev) {
            long long m = left[i];
            std::vector<long long> rest(left.begin(), left.end());
            rest.erase(rest.begin() + (long) i);
            total += go(m, true, rest);
        }
    }
    return total;
}

long long lineups(std::vector<long long> marks) {
    return go(0, false, marks);
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(lineups(from_json<std::vector<long long>>(args[0])));
}
