// Test-only known-correct C++ fixture for ember-isles.
long long isle_count(std::vector<std::vector<long long>> edges, long long n) {
    if (n < 0) {
        n = 0;
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
    long long count = 0;
    for (long long s = 0; s < n; s += 1) {
        if (seen[s]) {
            continue;
        }
        count += 1;
        seen[s] = 1;
        std::vector<long long> stack{s};
        while (!stack.empty()) {
            long long x = stack.back();
            stack.pop_back();
            for (long long y : adj[x]) {
                if (!seen[y]) {
                    seen[y] = 1;
                    stack.push_back(y);
                }
            }
        }
    }
    return count;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(isle_count(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<long long>(args[1])));
}
