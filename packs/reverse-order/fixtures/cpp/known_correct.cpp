// Test-only known-correct C++ fixture for reverse-order.
std::vector<long long> reverse_order(std::vector<long long> marks, std::vector<long long> nxt) {
    std::vector<long long> out;
    for (long long i = nxt.empty() ? -1 : 0; i != -1; i = nxt[(size_t) i]) { out.push_back(marks[(size_t) i]); }
    std::reverse(out.begin(), out.end());
    return out;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(reverse_order(
        from_json<std::vector<long long>>(args[0]),
        from_json<std::vector<long long>>(args[1])));
}
