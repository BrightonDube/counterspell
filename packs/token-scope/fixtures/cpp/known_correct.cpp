// Test-only known-correct C++ fixture for token-scope.
long long scope_ok(std::vector<long long> required, std::vector<long long> granted, long long expiry, long long now) {
    bool alive = now < expiry;
    bool covered = true;
    std::set<long long> g(granted.begin(), granted.end());
    for (long long s : required) if (!g.count(s)) covered = false;
    return alive && covered ? 1 : 0;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(scope_ok(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<long long>(args[2]), from_json<long long>(args[3])));
}
