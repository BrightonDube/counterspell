// Test-only known-correct C++ fixture for rune-gap.
static long long go(const std::vector<long long>& marks, size_t i, long long left) {
    if (left < 0) {
        return 0;
    }
    if (i == marks.size()) {
        return 1;
    }
    return go(marks, i + 1, left - marks[i]) + go(marks, i + 1, left);
}

long long gap_count(std::vector<long long> marks, long long cap) {
    return go(marks, 0, cap);
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(gap_count(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
