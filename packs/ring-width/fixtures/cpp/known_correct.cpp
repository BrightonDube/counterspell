// Test-only known-correct C++ fixture for ring-width.
long long widest_ring(std::vector<long long> vals, std::vector<long long> left, std::vector<long long> right) {
    long long n = (long long) vals.size(), best = 0;
    if (n == 0) return 0;
    std::map<long long, long long> counts;
    std::set<long long> seen;
    std::deque<std::pair<long long, long long>> queue = {{0, 0}};
    while (!queue.empty()) {
        auto cur = queue.front(); queue.pop_front();
        if (seen.count(cur.first)) continue;
        seen.insert(cur.first);
        counts[cur.second] += 1;
        if (counts[cur.second] > best) best = counts[cur.second];
        long long lo = cur.first < (long long) left.size() && left[cur.first] >= 0 && left[cur.first] < n ? left[cur.first] : -1;
        long long hi = cur.first < (long long) right.size() && right[cur.first] >= 0 && right[cur.first] < n ? right[cur.first] : -1;
        if (lo != -1) queue.push_back({lo, cur.second + 1});
        if (hi != -1) queue.push_back({hi, cur.second + 1});
    }
    return best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(widest_ring(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<std::vector<long long>>(args[2])));
}
