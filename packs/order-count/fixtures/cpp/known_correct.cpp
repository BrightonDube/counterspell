// Test-only known-correct C++ fixture for order-count.
static long long go_order(int n, int k, const std::vector<std::vector<int>>& adj, std::vector<int>& indeg) {
    if (k == n) {
        return 1;
    }
    long long total = 0;
    for (int v = 0; v < n; v += 1) {
        if (indeg[v] == 0) {
            indeg[v] = -1;
            for (int w : adj[v]) {
                indeg[w] -= 1;
            }
            total += go_order(n, k + 1, adj, indeg);
            for (int w : adj[v]) {
                indeg[w] += 1;
            }
            indeg[v] = 0;
        }
    }
    return total;
}

long long order_count(std::vector<std::vector<long long>> edges, long long n) {
    if (n < 0) {
        n = 0;
    }
    std::vector<std::vector<int>> adj(n);
    std::vector<int> indeg(n, 0);
    bool self_order = false;
    for (auto& e : edges) {
        int a = (int) e[0];
        int b = (int) e[1];
        if (!(a >= 0 && a < n && b >= 0 && b < n)) {
            continue;
        }
        if (a == b) {
            self_order = true;
            continue;
        }
        if (std::find(adj[a].begin(), adj[a].end(), b) == adj[a].end()) {
            adj[a].push_back(b);
            indeg[b] += 1;
        }
    }
    if (self_order) {
        return 0;
    }
    return go_order((int) n, 0, adj, indeg);
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(order_count(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<long long>(args[1])));
}
