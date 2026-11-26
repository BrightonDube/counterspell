// Test-only known-correct C++ fixture for gap-blocks.
long long block_count(std::vector<std::vector<long long>> spans) {
    if (spans.empty()) {
        return 0;
    }
    std::sort(spans.begin(), spans.end());
    long long blocks = 1;
    long long end = spans[0][1];
    for (size_t i = 1; i < spans.size(); i += 1) {
        long long a = spans[i][0], b = spans[i][1];
        if (a > end) {
            blocks += 1;
            end = b;
        } else {
            end = std::max(end, b);
        }
    }
    return blocks;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(block_count(from_json<std::vector<std::vector<long long>>>(args[0])));
}
