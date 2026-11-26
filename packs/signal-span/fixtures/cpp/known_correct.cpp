// Test-only known-correct C++ fixture for signal-span.
long long span_max(std::vector<std::vector<long long>> edges, long long n) {
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
    std::vector<long long> dist(n, -1);
    dist[0] = 0;
    std::vector<long long> queue{0};
    for (size_t i = 0; i < queue.size(); i += 1) {
        long long x = queue[i];
        for (long long y : adj[x]) {
            if (dist[y] < 0) {
                dist[y] = dist[x] + 1;
                queue.push_back(y);
            }
        }
    }
    return *std::max_element(dist.begin(), dist.end());
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(span_max(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<long long>(args[1])));
}
