// Test-only known-correct C++ fixture for route-count.
long long route_count(long long n, std::vector<std::vector<long long>> links) {
    if (n == 0) return 0;
    int m = (int)n;
    std::vector<std::vector<int>> adj(m);
    for (auto& e : links) { if (0 <= e[0] && e[0] < n && 0 <= e[1] && e[1] < n) { adj[e[0]].push_back((int)e[1]); adj[e[1]].push_back((int)e[0]); } }
    std::vector<bool> seen(m, false);
    std::vector<int> q{0};
    seen[0] = true;
    for (size_t i = 0; i < q.size(); i++)
        for (int v : adj[q[i]])
            if (!seen[v]) { seen[v] = true; q.push_back(v); }
    long long count = 0;
    for (bool s : seen) count += s;
    return count - 1;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(route_count(from_json<long long>(args[0]), from_json<std::vector<std::vector<long long>>>(args[1])));
}
