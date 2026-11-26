// Test-only known-correct C++ fixture for twin-walls.
long long twin_walls(std::vector<std::vector<long long>> edges, long long n) {
    if (n < 0) {
        n = 0;
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
    std::vector<int> colour(n, -1);
    for (long long s = 0; s < n; s += 1) {
        if (colour[s] != -1) {
            continue;
        }
        colour[s] = 0;
        std::vector<long long> stack{s};
        while (!stack.empty()) {
            long long x = stack.back();
            stack.pop_back();
            for (long long y : adj[x]) {
                if (colour[y] == -1) {
                    colour[y] = 1 - colour[x];
                    stack.push_back(y);
                } else if (colour[y] == colour[x]) {
                    return 0;
                }
            }
        }
    }
    return 1;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(twin_walls(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<long long>(args[1])));
}
