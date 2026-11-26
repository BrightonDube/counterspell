// Test-only known-correct C++ fixture for ember-diameter.
long long tree_diameter(std::vector<long long> vals, std::vector<long long> left, std::vector<long long> right) {
    long long n = (long long) vals.size(), best = 0;
    if (n == 0) return 0;
    std::vector<long long> heights(n, -1);
    std::set<long long> seen;
    std::vector<std::pair<long long, bool>> stack = {{0, false}};
    while (!stack.empty()) {
        auto cur = stack.back(); stack.pop_back();
        long long i = cur.first;
        if (seen.count(i)) continue;
        if (!cur.second) {
            stack.push_back({i, true});
            for (auto* arr : {&left, &right}) {
                long long k = i < (long long) arr->size() && (*arr)[i] >= 0 && (*arr)[i] < n ? (*arr)[i] : -1;
                if (k != -1) stack.push_back({k, false});
            }
            continue;
        }
        seen.insert(i);
        long long lo = i < (long long) left.size() && left[i] >= 0 && left[i] < n ? left[i] : -1;
        long long hi = i < (long long) right.size() && right[i] >= 0 && right[i] < n ? right[i] : -1;
        long long lh = lo != -1 && heights[lo] >= 0 ? heights[lo] + 1 : 0;
        long long rh = hi != -1 && heights[hi] >= 0 ? heights[hi] + 1 : 0;
        if (lh + rh > best) best = lh + rh;
        heights[i] = std::max(lh, rh);
    }
    return best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(tree_diameter(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<std::vector<long long>>(args[2])));
}
