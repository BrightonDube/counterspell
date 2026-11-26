// Test-only known-correct C++ fixture for bst-ward.
long long bst_check(std::vector<long long> vals, std::vector<long long> left, std::vector<long long> right) {
    long long n = (long long) vals.size();
    if (n == 0) return 1;
    long long lo0 = LLONG_MIN / 2, hi0 = LLONG_MAX / 2;
    std::set<std::tuple<long long, long long, long long>> seen;
    std::vector<std::tuple<long long, long long, long long>> work = {{0, lo0, hi0}};
    while (!work.empty()) {
        auto cur = work.back(); work.pop_back();
        long long i = std::get<0>(cur), lo = std::get<1>(cur), hi = std::get<2>(cur);
        if (!(lo < vals[i] && vals[i] < hi)) return 0;
        if (seen.count(cur)) continue;
        seen.insert(cur);
        long long l = i < (long long) left.size() && left[i] >= 0 && left[i] < n ? left[i] : -1;
        long long r = i < (long long) right.size() && right[i] >= 0 && right[i] < n ? right[i] : -1;
        if (l != -1) work.push_back({l, lo, vals[i]});
        if (r != -1) work.push_back({r, vals[i], hi});
    }
    return 1;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(bst_check(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<std::vector<long long>>(args[2])));
}
