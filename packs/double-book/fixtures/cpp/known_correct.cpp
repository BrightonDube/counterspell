// Test-only known-correct C++ fixture for double-book.
long long room_count(std::vector<std::vector<long long>> spans) {
    std::vector<std::pair<long long, long long>> evs;
    for (auto& s : spans) {
        evs.push_back({s[0], 1});
        evs.push_back({s[1], -1});
    }
    std::sort(evs.begin(), evs.end(), [](auto& x, auto& y) {
        return x.first != y.first ? x.first < y.first : x.second > y.second;
    });
    long long cur = 0;
    long long best = 0;
    for (auto& e : evs) {
        cur += e.second;
        best = std::max(best, cur);
    }
    return best;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(room_count(from_json<std::vector<std::vector<long long>>>(args[0])));
}
