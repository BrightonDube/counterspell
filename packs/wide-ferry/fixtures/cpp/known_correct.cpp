// Test-only known-correct C++ fixture for wide-ferry.
long long ferry_load(std::vector<std::vector<long long>> edges, std::vector<long long> weights, long long n) {
    if (n <= 1) {
        return n == 1 ? 0 : -1;
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
    std::vector<long long> best(n, -1);
    best[0] = std::numeric_limits<long long>::max();
    std::priority_queue<std::pair<long long, long long>> pq;
    pq.push({best[0], 0});
    while (!pq.empty()) {
        auto top = pq.top();
        pq.pop();
        long long cap = top.first;
        long long u = top.second;
        if (cap < best[u]) {
            continue;
        }
        for (auto& e : adj[u]) {
            long long v = e.first;
            long long cand = std::min(cap, e.second);
            if (cand > best[v]) {
                best[v] = cand;
                pq.push({cand, v});
            }
        }
    }
    return best[n - 1];
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(ferry_load(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<long long>(args[2])));
}
