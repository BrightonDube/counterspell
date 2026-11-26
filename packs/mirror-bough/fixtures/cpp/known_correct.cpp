// Test-only known-correct C++ fixture for mirror-bough.
static long long mirror_kid(std::vector<long long>& arr, long long i, long long n) {
    return i < (long long) arr.size() && arr[i] >= 0 && arr[i] < n ? arr[i] : -1;
}
long long mirror_check(std::vector<long long> vals, std::vector<long long> left, std::vector<long long> right) {
    long long n = (long long) vals.size();
    if (n == 0) return 1;
    std::set<std::pair<long long, long long>> seen;
    std::vector<std::pair<long long, long long>> work = {{mirror_kid(left, 0, n), mirror_kid(right, 0, n)}};
    while (!work.empty()) {
        auto cur = work.back(); work.pop_back();
        if (seen.count(cur)) continue;
        seen.insert(cur);
        if (cur.first == -1 && cur.second == -1) continue;
        if (cur.first == -1 || cur.second == -1 || vals[cur.first] != vals[cur.second]) return 0;
        work.push_back({mirror_kid(left, cur.first, n), mirror_kid(right, cur.second, n)});
        work.push_back({mirror_kid(right, cur.first, n), mirror_kid(left, cur.second, n)});
    }
    return 1;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(mirror_check(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<std::vector<long long>>(args[2])));
}
