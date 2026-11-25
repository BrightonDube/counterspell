// Test-only known-correct C++ fixture for tide-table (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
long long first_drop(std::vector<long long> heights) {
    for (std::size_t index = 1; index < heights.size(); index += 1) {
        if (heights[index] < heights[index - 1]) return index;
    }
    return -1;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(first_drop(from_json<std::vector<long long>>(args[0])));
}
