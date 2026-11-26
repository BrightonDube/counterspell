// Test-only known-correct C++ fixture for wood-span.
static int find_root(std::vector<int>& parent, int x) {
    while (parent[x] != x) {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    return x;
}

long long span_toll(std::vector<std::vector<long long>> edges, std::vector<long long> weights, long long n) {
    if (n <= 0) {
        return 0;
    }
    std::vector<int> parent(n);
    for (int i = 0; i < n; i += 1) {
        parent[i] = i;
    }
    std::vector<std::tuple<long long, int, int>> arcs;
    size_t len = std::min(edges.size(), weights.size());
    for (size_t k = 0; k < len; k += 1) {
        int a = (int) edges[k][0];
        int b = (int) edges[k][1];
        long long c = weights[k];
        if (a >= 0 && a < n && b >= 0 && b < n && a != b) {
            arcs.emplace_back(c, a, b);
        }
    }
    std::sort(arcs.begin(), arcs.end());
    long long total = 0;
    long long used = 0;
    for (auto& e : arcs) {
        int ra = find_root(parent, std::get<1>(e));
        int rb = find_root(parent, std::get<2>(e));
        if (ra != rb) {
            parent[ra] = rb;
            total += std::get<0>(e);
            used += 1;
        }
    }
    return used == n - 1 ? total : -1;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(span_toll(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<long long>(args[2])));
}
