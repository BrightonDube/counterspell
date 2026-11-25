// Test-only known-correct C++ fixture for shared-link.
long long shared_link(std::vector<long long> nxt, long long ha, long long hb) {
    std::set<long long> seen;
    for (long long i = ha; i != -1; i = nxt[(size_t) i]) { seen.insert(i); }
    long long j = hb;
    while (j != -1 && !seen.count(j)) { j = nxt[(size_t) j]; }
    return j;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(shared_link(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1]),
        from_json<long long>(args[2])));
}
