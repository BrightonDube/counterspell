// Test-only known-correct C++ fixture for sched-rite.
static bool visit_rite(std::map<long long, std::vector<long long>>& adj,
                       std::map<long long, int>& state, long long u) {
    state[u] = 1;
    for (long long v : adj[u]) {
        int s = state.count(v) ? state[v] : 0;
        if (s == 1) {
            return true;
        }
        if (s == 0 && visit_rite(adj, state, v)) {
            return true;
        }
    }
    state[u] = 2;
    return false;
}

long long sched_rite(std::vector<std::vector<long long>> pairs) {
    std::map<long long, std::vector<long long>> adj;
    for (auto& p : pairs) {
        adj[p[0]].push_back(p[1]);
    }
    std::map<long long, int> state;
    for (auto& [u, _] : adj) {
        if (!state.count(u) && visit_rite(adj, state, u)) {
            return 0;
        }
    }
    return 1;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(sched_rite(from_json<std::vector<std::vector<long long>>>(args[0])));
}
