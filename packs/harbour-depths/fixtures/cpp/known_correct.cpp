// Test-only known-correct C++ fixture for harbour-depths (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
long long deepest_water(std::vector<long long> heights) {
    if (heights.empty()) return 0;
    std::size_t left = 0, right = heights.size() - 1;
    long long deepest = 0;
    while (left < right) {
        long long held = std::min(heights[left], heights[right])
            * static_cast<long long>(right - left);
        if (held > deepest) deepest = held;
        if (heights[left] <= heights[right]) left += 1;
        else right -= 1;
    }
    return deepest;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(deepest_water(from_json<std::vector<long long>>(args[0])));
}
