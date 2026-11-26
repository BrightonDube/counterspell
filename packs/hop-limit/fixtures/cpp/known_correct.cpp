// Test-only known-correct C++ fixture for hop-limit.
long long hop_limit(long long n, std::vector<std::vector<long long>> links, long long ttl) {
    if (n == 0) return 0;
    int m = (int)n;
    std::vector<std::vector<int>> adj(m);
    for (auto& e : links) { if (0 <= e[0] && e[0] < n && 0 <= e[1] && e[1] < n) { adj[e[0]].push_back((int)e[1]); adj[e[1]].push_back((int)e[0]); } }
    std::vector<int> dist(m, -1);
    dist[0] = 0;
    std::vector<int> q{0};
    for (size_t i = 0; i < q.size(); i++) {
        int u = q[i];
        if (dist[u] >= ttl) continue;
        for (int v : adj[u]) {
            if (dist[v] < 0) { dist[v] = dist[u] + 1; q.push_back(v); }
        }
    }
    long long count = 0;
    for (int d : dist) count += d >= 0;
    return count - 1;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(hop_limit(from_json<long long>(args[0]), from_json<std::vector<std::vector<long long>>>(args[1]), from_json<long long>(args[2])));
}
