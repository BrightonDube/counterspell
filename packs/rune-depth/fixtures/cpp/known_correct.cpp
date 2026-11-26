// Test-only known-correct C++ fixture for rune-depth.
static bool dfs_depth(long long u, const std::vector<std::vector<long long>>& adj,
                      std::vector<int>& state, std::vector<long long>& memo) {
    state[u] = 1;
    long long best = 0;
    for (long long w : adj[u]) {
        if (state[w] == 1) {
            return false;
        }
        if (state[w] == 0 && !dfs_depth(w, adj, state, memo)) {
            return false;
        }
        best = std::max(best, memo[w]);
    }
    state[u] = 2;
    memo[u] = best + 1;
    return true;
}

long long rune_depth(std::vector<std::vector<long long>> edges, long long n) {
    if (n < 0) {
        n = 0;
    }
    std::vector<std::vector<long long>> adj(n);
    bool self_order = false;
    for (auto& e : edges) {
        long long a = e[0];
        long long b = e[1];
        if (!(a >= 0 && a < n && b >= 0 && b < n)) {
            continue;
        }
        if (a == b) {
            self_order = true;
            continue;
        }
        if (std::find(adj[a].begin(), adj[a].end(), b) == adj[a].end()) {
            adj[a].push_back(b);
        }
    }
    if (self_order) {
        return 0;
    }
    std::vector<int> state(n, 0);
    std::vector<long long> memo(n, 0);
    for (long long v = 0; v < n; v += 1) {
        if (state[v] == 0 && !dfs_depth(v, adj, state, memo)) {
            return 0;
        }
    }
    long long best = 0;
    for (long long m : memo) {
        best = std::max(best, m);
    }
    return best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(rune_depth(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<long long>(args[1])));
}
