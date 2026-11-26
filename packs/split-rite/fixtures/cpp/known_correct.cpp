// Test-only known-correct C++ fixture for split-rite.
long long split_rite(long long n, std::vector<std::vector<long long>> links, std::vector<long long> down) {
    int m = (int)n;
    std::set<long long> dead(down.begin(), down.end());
    std::vector<std::vector<int>> adj(m);
    for (size_t i = 0; i < links.size(); i++) {
        if (dead.count((long long)i)) continue;
        if (0 <= links[i][0] && links[i][0] < n && 0 <= links[i][1] && links[i][1] < n) {
            adj[links[i][0]].push_back((int)links[i][1]);
            adj[links[i][1]].push_back((int)links[i][0]);
        }
    }
    std::vector<bool> seen(m, false);
    long long quorum = 0;
    for (int s = 0; s < m; s++) {
        if (seen[s]) continue;
        long long size = 0;
        std::vector<int> q{s};
        seen[s] = true;
        for (size_t i = 0; i < q.size(); i++) {
            size++;
            for (int v : adj[q[i]])
                if (!seen[v]) { seen[v] = true; q.push_back(v); }
        }
        if (size * 2 > n) quorum++;
    }
    return quorum;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(split_rite(from_json<long long>(args[0]), from_json<std::vector<std::vector<long long>>>(args[1]), from_json<std::vector<long long>>(args[2])));
}
