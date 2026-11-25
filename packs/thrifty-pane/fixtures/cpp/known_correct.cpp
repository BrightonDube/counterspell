// Test-only known-correct C++ fixture for thrifty-pane.
long long longest_thrifty(std::string pane) {
    long long counts[3] = {0, 0, 0};
    int distinct = 0;
    std::size_t left = 0;
    long long best = 0;
    for (std::size_t right = 0; right < pane.size(); right += 1) {
        int slot = pane[right] - 'a';
        if (counts[slot] == 0) {
            distinct += 1;
        }
        counts[slot] += 1;
        while (distinct > 2) {
            int edge = pane[left] - 'a';
            counts[edge] -= 1;
            if (counts[edge] == 0) {
                distinct -= 1;
            }
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
    return to_json(longest_thrifty(from_json<std::string>(args[0])));
}
