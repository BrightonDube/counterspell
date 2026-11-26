// Test-only known-correct C++ fixture for ember-road.
long long road_toll(std::vector<std::vector<long long>> edges, std::vector<long long> weights, long long n) {
    if (n <= 0) {
        return -1;
    }
    std::vector<std::vector<std::pair<long long, long long>>> adj(n);
    size_t len = std::min(edges.size(), weights.size());
    for (size_t k = 0; k < len; k += 1) {
        long long a = edges[k][0];
        long long b = edges[k][1];
        long long c = weights[k];
        if (a >= 0 && a < n && b >= 0 && b < n) {
            adj[a].push_back({b, c});
            adj[b].push_back({a, c});
        }
    }
    const long long INF = std::numeric_limits<long long>::max();
    std::vector<long long> dist(n, INF);
    dist[0] = 0;
    std::priority_queue<std::pair<long long, long long>, std::vector<std::pair<long long, long long>>, std::greater<>> pq;
    pq.push({0, 0});
    while (!pq.empty()) {
        auto top = pq.top();
        pq.pop();
        long long du = top.first;
        long long u = top.second;
        if (du > dist[u]) {
            continue;
        }
        for (auto& e : adj[u]) {
            long long v = e.first;
            long long nd = du + e.second;
            if (nd < dist[v]) {
                dist[v] = nd;
                pq.push({nd, v});
            }
        }
    }
    return dist[n - 1] == INF ? -1 : dist[n - 1];
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(road_toll(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<long long>(args[2])));
}
