// Test-only known-correct C++ fixture for closest-quota (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
long long nearest_pair(std::vector<long long> marks, long long quota) {
    if (marks.size() < 2) return 0;
    std::size_t left = 0, right = marks.size() - 1;
    long long best = 0;
    bool seen = false;
    while (left < right) {
        long long total = marks[left] + marks[right];
        long long gap = total > quota ? total - quota : quota - total;
        long long best_gap = best > quota ? best - quota : quota - best;
        if (!seen || gap < best_gap || (gap == best_gap && total < best)) {
            best = total;
            seen = true;
        }
        if (total < quota) left += 1;
        else right -= 1;
    }
    return best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(nearest_pair(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1])));
}
