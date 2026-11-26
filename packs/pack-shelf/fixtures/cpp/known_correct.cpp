// Test-only known-correct C++ fixture for pack-shelf.
long long pack_worth(std::vector<std::vector<long long>> items, long long cap) {
    std::vector<long long> d(cap + 1, 0);
    for (auto& item : items) {
        long long w = item[0];
        long long v = item[1];
        for (long long i = cap; i >= w; i -= 1) {
            d[i] = std::max(d[i], d[i - w] + v);
        }
    }
    return d[cap];
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(pack_worth(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<long long>(args[1])));
}
