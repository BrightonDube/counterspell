// Test-only known-correct C++ fixture for kindred-path.
long long ember_path(std::vector<long long> vals, std::vector<long long> left, std::vector<long long> right, long long target) {
    long long n = (long long) vals.size();
    if (n == 0) return 0;
    std::set<std::pair<long long, long long>> seen;
    std::vector<std::pair<long long, long long>> work = {{0, vals[0]}};
    while (!work.empty()) {
        auto cur = work.back(); work.pop_back();
        long long i = cur.first, total = cur.second;
        if (seen.count(cur)) continue;
        seen.insert(cur);
        long long lo = i < (long long) left.size() && left[i] >= 0 && left[i] < n ? left[i] : -1;
        long long hi = i < (long long) right.size() && right[i] >= 0 && right[i] < n ? right[i] : -1;
        if (lo == -1 && hi == -1) {
            if (total == target) return 1;
            continue;
        }
        if (lo != -1) work.push_back({lo, total + vals[lo]});
        if (hi != -1) work.push_back({hi, total + vals[hi]});
    }
    return 0;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(ember_path(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<std::vector<long long>>(args[2]), from_json<long long>(args[3])));
}
