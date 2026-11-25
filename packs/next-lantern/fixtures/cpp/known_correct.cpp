// Test-only known-correct C++ fixture for next-lantern.
std::vector<long long> next_taller(std::vector<long long> heights) {
    std::vector<long long> answer(heights.size(), 0);
    std::vector<long long> waiting;
    for (long long index = (long long)heights.size() - 1; index >= 0; index -= 1) {
        while (!waiting.empty() && heights[waiting.back()] <= heights[index]) {
            waiting.pop_back();
        }
        if (!waiting.empty()) {
            answer[index] = waiting.back() - index;
        }
        waiting.push_back(index);
    }
    return answer;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(next_taller(from_json<std::vector<long long>>(args[0])));
}
