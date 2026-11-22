// Test-only known-correct C++ fixture for count-pairs (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
long long count_pairs(std::vector<long long> values, long long target) {
    long long pairs = 0;
    for (std::size_t left = 0; left < values.size(); left += 1) {
        for (std::size_t right = left + 1; right < values.size(); right += 1) {
            if (values[left] + values[right] == target) {
                pairs += 1;
            }
        }
    }
    return pairs;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(count_pairs(from_json<std::vector<long long>>(args[0]),
                               from_json<long long>(args[1])));
}
