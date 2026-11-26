// Test-only known-correct C++ fixture for light-isles.
static int find_root(std::vector<int>& parent, int x) {
    while (parent[x] != x) {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    return x;
}

long long light_isles(std::vector<std::vector<long long>> edges, std::vector<long long> weights, long long n, long long t) {
    if (n < 0) {
        n = 0;
    }
    std::vector<int> parent(n);
    for (int i = 0; i < n; i += 1) {
        parent[i] = i;
    }
    size_t len = std::min(edges.size(), weights.size());
    for (size_t k = 0; k < len; k += 1) {
        int a = (int) edges[k][0];
        int b = (int) edges[k][1];
        long long c = weights[k];
        if (c <= t && a >= 0 && a < n && b >= 0 && b < n) {
            int ra = find_root(parent, a);
            int rb = find_root(parent, b);
            if (ra != rb) {
                parent[ra] = rb;
            }
        }
    }
    std::set<int> roots;
    for (int i = 0; i < n; i += 1) {
        roots.insert(find_root(parent, i));
    }
    return (long long) roots.size();
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(light_isles(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<long long>(args[2]), from_json<long long>(args[3])));
}
