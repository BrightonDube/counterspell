// Test-only known-correct C++ fixture for flood-line (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
long long trapped_water(std::vector<long long> heights) {
    if (heights.size() < 3) return 0;
    std::size_t left = 0, right = heights.size() - 1;
    long long leftBound = 0, rightBound = 0, water = 0;
    while (left < right) {
        if (heights[left] <= heights[right]) {
            leftBound = std::max(leftBound, heights[left]);
            water += leftBound - heights[left];
            left += 1;
        } else {
            rightBound = std::max(rightBound, heights[right]);
            water += rightBound - heights[right];
            right -= 1;
        }
    }
    return water;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(trapped_water(from_json<std::vector<long long>>(args[0])));
}
