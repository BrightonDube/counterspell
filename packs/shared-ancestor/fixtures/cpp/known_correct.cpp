// Test-only known-correct C++ fixture for shared-ancestor.
long long first_ancestor(std::vector<long long> vals, std::vector<long long> left, std::vector<long long> right, long long a, long long b) {
    long long n = (long long) vals.size();
    if (n == 0 || a < 0 || a >= n || b < 0 || b >= n) return -1;
    std::map<long long, long long> parent;
    std::set<long long> seen;
    std::vector<long long> stack = {0};
    while (!stack.empty()) {
        long long i = stack.back(); stack.pop_back();
        if (seen.count(i)) continue;
        seen.insert(i);
        for (auto* arr : {&left, &right}) {
            long long k = i < (long long) arr->size() && (*arr)[i] >= 0 && (*arr)[i] < n ? (*arr)[i] : -1;
            if (k != -1 && !parent.count(k)) { parent[k] = i; stack.push_back(k); }
        }
    }
    if (!seen.count(a) || !seen.count(b)) return -1;
    std::set<long long> anc;
    for (long long x = a; x != -1; x = parent.count(x) ? parent[x] : -1) anc.insert(x);
    for (long long x = b; x != -1; x = parent.count(x) ? parent[x] : -1) {
        if (anc.count(x)) return x;
    }
    return -1;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(first_ancestor(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<std::vector<long long>>(args[2]), from_json<long long>(args[3]), from_json<long long>(args[4])));
}
