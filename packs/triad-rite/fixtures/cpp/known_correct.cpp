// Test-only known-correct C++ fixture for triad-rite (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
long long triads_at_quota(std::vector<long long> marks, long long quota) {
    long long count = 0;
    std::size_t n = marks.size();
    for (std::size_t i = 0; i + 2 < n; i += 1) {
        long long target = quota - marks[i];
        std::size_t left = i + 1, right = n - 1;
        while (left < right) {
            long long pair = marks[left] + marks[right];
            if (pair < target) {
                left += 1;
            } else if (pair > target) {
                right -= 1;
            } else if (marks[left] == marks[right]) {
                long long span = static_cast<long long>(right - left + 1);
                count += span * (span - 1) / 2;
                break;
            } else {
                std::size_t lo = 1;
                while (left + lo < right && marks[left + lo] == marks[left]) lo += 1;
                std::size_t hi = 1;
                while (right - hi > left && marks[right - hi] == marks[right]) hi += 1;
                count += static_cast<long long>(lo * hi);
                left += lo;
                right -= hi;
            }
        }
    }
    return count;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(triads_at_quota(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1])));
}
