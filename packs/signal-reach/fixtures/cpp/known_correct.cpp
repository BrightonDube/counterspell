// Test-only known-correct C++ fixture for signal-reach.
long long reach_count(std::vector<std::vector<long long>> edges, long long n) {
    if (n <= 0) {
        return 0;
    }
    std::vector<std::vector<long long>> adj(n);
    for (auto& e : edges) {
        long long a = e[0];
        long long b = e[1];
        if (a >= 0 && a < n && b >= 0 && b < n) {
            adj[a].push_back(b);
            adj[b].push_back(a);
        }
    }
    std::vector<char> seen(n, 0);
    seen[0] = 1;
    std::vector<long long> stack{0};
    long long count = 0;
    while (!stack.empty()) {
        long long x = stack.back();
        stack.pop_back();
        count += 1;
        for (long long y : adj[x]) {
            if (!seen[y]) {
                seen[y] = 1;
                stack.push_back(y);
            }
        }
    }
    return count;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(reach_count(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<long long>(args[1])));
}
