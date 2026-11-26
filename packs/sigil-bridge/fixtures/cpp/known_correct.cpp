// Test-only known-correct C++ fixture for sigil-bridge.
static void dfs_bridge(long long u, long long entry_arc,
                       const std::vector<std::vector<std::pair<long long, long long>>>& adj,
                       std::vector<long long>& disc, std::vector<long long>& low,
                       long long& stamp, long long& bridges) {
    disc[u] = stamp;
    low[u] = stamp;
    stamp += 1;
    for (auto& e : adj[u]) {
        long long v = e.first;
        long long arc = e.second;
        if (arc == entry_arc) {
            continue;
        }
        if (disc[v] == -1) {
            dfs_bridge(v, arc, adj, disc, low, stamp, bridges);
            low[u] = std::min(low[u], low[v]);
            if (low[v] > disc[u]) {
                bridges += 1;
            }
        } else {
            low[u] = std::min(low[u], disc[v]);
        }
    }
}

long long bridge_count(std::vector<std::vector<long long>> edges, long long n) {
    if (n < 0) {
        n = 0;
    }
    std::vector<std::vector<std::pair<long long, long long>>> adj(n);
    long long arc = 0;
    for (auto& e : edges) {
        long long a = e[0];
        long long b = e[1];
        if (a >= 0 && a < n && b >= 0 && b < n && a != b) {
            adj[a].push_back({b, arc});
            adj[b].push_back({a, arc});
            arc += 1;
        }
    }
    std::vector<long long> disc(n, -1);
    std::vector<long long> low(n, 0);
    long long stamp = 0;
    long long bridges = 0;
    for (long long s = 0; s < n; s += 1) {
        if (disc[s] == -1) {
            dfs_bridge(s, -1, adj, disc, low, stamp, bridges);
        }
    }
    return bridges;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(bridge_count(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<long long>(args[1])));
}
