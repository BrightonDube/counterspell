// Test-only known-correct C++ fixture for miss-count.
long long miss_count(std::vector<long long> marks, long long c) {
    std::vector<long long> slots(c, -1);
    long long misses = 0;
    for (long long m : marks) {
        long long line = m % c, tag = m / c;
        if (slots[line] != tag) { misses += 1; slots[line] = tag; }
    }
    return misses;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(miss_count(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
