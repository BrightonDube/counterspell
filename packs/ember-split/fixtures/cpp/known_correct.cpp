// Test-only known-correct C++ fixture for ember-split.
static long long go(const std::vector<long long>& marks, size_t i, long long need) {
    if (need == 0) {
        return 1;
    }
    if (i == marks.size()) {
        return 0;
    }
    return go(marks, i + 1, need - marks[i]) + go(marks, i + 1, need);
}

long long split_count(std::vector<long long> marks) {
    long long total = 0;
    for (long long m : marks) {
        total += m;
    }
    if (total % 2 != 0) {
        return 0;
    }
    long long half = total / 2;
    if (half == 0) {
        return ((1LL << marks.size()) - 2) / 2;
    }
    return go(marks, 0, half) / 2;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(split_count(from_json<std::vector<long long>>(args[0])));
}
