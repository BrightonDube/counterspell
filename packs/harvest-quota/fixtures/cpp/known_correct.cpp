// Test-only known-correct C++ fixture for harvest-quota.
long long longest_harvest(std::vector<long long> marks, long long quota) {
    std::size_t left = 0;
    long long total = 0;
    long long best = 0;
    for (std::size_t right = 0; right < marks.size(); right += 1) {
        total += marks[right];
        while (total > quota) {
            total -= marks[left];
            left += 1;
        }
        long long span = (long long) (right - left + 1);
        if (span > best) {
            best = span;
        }
    }
    return best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(longest_harvest(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1])));
}
