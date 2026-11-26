// Test-only known-correct C++ fixture for ebb-road.
long long ebb_toll(std::vector<std::vector<long long>> edges, std::vector<long long> weights, long long n) {
    const long long RING = -1000000000000000LL;
    if (n <= 0) {
        return -1;
    }
    std::vector<std::tuple<int, int, long long>> arcs;
    size_t len = std::min(edges.size(), weights.size());
    for (size_t k = 0; k < len; k += 1) {
        int a = (int) edges[k][0];
        int b = (int) edges[k][1];
        long long c = weights[k];
        if (a >= 0 && a < n && b >= 0 && b < n) {
            arcs.emplace_back(a, b, c);
        }
    }
    const long long INF = std::numeric_limits<long long>::max() / 4;
    std::vector<long long> dist(n, INF);
    dist[0] = 0;
    for (int i = 0; i < n - 1; i += 1) {
        for (auto& e : arcs) {
            int a = std::get<0>(e);
            int b = std::get<1>(e);
            long long c = std::get<2>(e);
            if (dist[a] < INF && dist[a] + c < dist[b]) {
                dist[b] = dist[a] + c;
            }
        }
    }
    std::vector<char> marked(n, 0);
    for (int i = 0; i < n; i += 1) {
        for (auto& e : arcs) {
            int a = std::get<0>(e);
            int b = std::get<1>(e);
            long long c = std::get<2>(e);
            if ((dist[a] < INF && dist[a] + c < dist[b]) || marked[a]) {
                marked[b] = 1;
            }
        }
    }
    if (marked[n - 1]) {
        return RING;
    }
    return dist[n - 1] >= INF ? -1 : dist[n - 1];
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(ebb_toll(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<long long>(args[2])));
}
