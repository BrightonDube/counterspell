// Test-only known-correct C++ fixture for toll-hop.
long long hop_toll(std::vector<std::vector<long long>> edges, std::vector<long long> weights, long long n, long long k) {
    if (n <= 0 || k < 0) {
        return -1;
    }
    std::vector<std::vector<std::pair<long long, long long>>> adj(n);
    size_t len = std::min(edges.size(), weights.size());
    for (size_t i = 0; i < len; i += 1) {
        long long a = edges[i][0];
        long long b = edges[i][1];
        long long c = weights[i];
        if (a >= 0 && a < n && b >= 0 && b < n) {
            adj[a].push_back({b, c});
            adj[b].push_back({a, c});
        }
    }
    const long long INF = std::numeric_limits<long long>::max();
    std::vector<std::vector<long long>> dist(n, std::vector<long long>(k + 1, INF));
    dist[0][0] = 0;
    std::priority_queue<std::tuple<long long, long long, long long>, std::vector<std::tuple<long long, long long, long long>>, std::greater<>> pq;
    pq.push({0, 0, 0});
    while (!pq.empty()) {
        auto top = pq.top();
        pq.pop();
        long long du = std::get<0>(top);
        long long u = std::get<1>(top);
        long long h = std::get<2>(top);
        if (du > dist[u][h]) {
            continue;
        }
        for (auto& e : adj[u]) {
            long long v = e.first;
            if (h + 1 <= k && du + e.second < dist[v][h + 1]) {
                dist[v][h + 1] = du + e.second;
                pq.push({dist[v][h + 1], v, h + 1});
            }
        }
    }
    long long best = INF;
    for (long long d : dist[n - 1]) {
        best = std::min(best, d);
    }
    return best == INF ? -1 : best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(hop_toll(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<long long>(args[2]), from_json<long long>(args[3])));
}
