// Test-only known-correct C++ fixture for canopy-height.
long long tree_height(std::vector<long long> vals, std::vector<long long> left, std::vector<long long> right) {
    long long n = (long long) vals.size(), best = 0;
    if (n == 0) return 0;
    std::set<long long> seen;
    std::vector<std::pair<long long, long long>> stack = {{0, 1}};
    while (!stack.empty()) {
        auto cur = stack.back(); stack.pop_back();
        if (seen.count(cur.first)) continue;
        seen.insert(cur.first);
        if (cur.second > best) best = cur.second;
        long long lo = cur.first < (long long) left.size() && left[cur.first] >= 0 && left[cur.first] < n ? left[cur.first] : -1;
        long long hi = cur.first < (long long) right.size() && right[cur.first] >= 0 && right[cur.first] < n ? right[cur.first] : -1;
        if (lo != -1) stack.push_back({lo, cur.second + 1});
        if (hi != -1) stack.push_back({hi, cur.second + 1});
    }
    return best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(tree_height(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<std::vector<long long>>(args[2])));
}
