// Test-only known-correct C++ fixture for reorder-embers.
std::vector<long long> reorder_embers(std::vector<long long> marks, std::vector<long long> nxt) {
    std::vector<long long> order;
    for (long long i = nxt.empty() ? -1 : 0; i != -1; i = nxt[(size_t) i]) { order.push_back(marks[(size_t) i]); }
    size_t half = (order.size() + 1) / 2;
    std::vector<long long> back(order.begin() + (long long) half, order.end());
    std::reverse(back.begin(), back.end());
    std::vector<long long> out;
    for (size_t k = 0; k < half; k += 1) {
        out.push_back(order[k]);
        if (k < back.size()) { out.push_back(back[k]); }
    }
    return out;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(reorder_embers(
        from_json<std::vector<long long>>(args[0]),
        from_json<std::vector<long long>>(args[1])));
}
