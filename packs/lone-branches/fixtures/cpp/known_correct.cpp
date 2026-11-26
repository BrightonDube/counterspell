// Test-only known-correct C++ fixture for lone-branches.
long long single_children(std::vector<long long> vals, std::vector<long long> left, std::vector<long long> right) {
    long long n = (long long) vals.size(), count = 0;
    std::set<long long> seen;
    std::vector<long long> stack;
    if (n > 0) stack.push_back(0);
    while (!stack.empty()) {
        long long i = stack.back(); stack.pop_back();
        if (seen.count(i)) continue;
        seen.insert(i);
        long long lo = i < (long long) left.size() && left[i] >= 0 && left[i] < n ? left[i] : -1;
        long long hi = i < (long long) right.size() && right[i] >= 0 && right[i] < n ? right[i] : -1;
        if ((lo == -1) != (hi == -1)) count += 1;
        if (lo != -1) stack.push_back(lo);
        if (hi != -1) stack.push_back(hi);
    }
    return count;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(single_children(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<std::vector<long long>>(args[2])));
}
