// Test-only known-correct C++ fixture for cycle-ward.
static int find_root(std::vector<int>& parent, int x) {
    while (parent[x] != x) {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    return x;
}

long long cycle_check(std::vector<std::vector<long long>> edges, long long n) {
    if (n < 0) {
        n = 0;
    }
    std::vector<int> parent(n);
    for (int i = 0; i < n; i += 1) {
        parent[i] = i;
    }
    for (auto& e : edges) {
        int a = (int) e[0];
        int b = (int) e[1];
        if (!(a >= 0 && a < n && b >= 0 && b < n)) {
            continue;
        }
        if (a == b) {
            return 1;
        }
        int ra = find_root(parent, a);
        int rb = find_root(parent, b);
        if (ra == rb) {
            return 1;
        }
        parent[ra] = rb;
    }
    return 0;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(cycle_check(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<long long>(args[1])));
}
