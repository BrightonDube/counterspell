// Test-only known-correct C++ fixture for drain-order.
std::vector<long long> drain_order(std::vector<long long> events) {
    std::vector<std::pair<long long, long long>> pairs;
    for (size_t i = 0; i + 1 < events.size(); i += 2) {
        pairs.emplace_back(events[i], events[i + 1]);
    }
    std::sort(pairs.begin(), pairs.end());
    std::vector<long long> ans;
    for (const auto& p : pairs) {
        ans.push_back(p.second);
    }
    return ans;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(drain_order(from_json<std::vector<long long>>(args[0])));
}
