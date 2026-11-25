// Test-only known-correct C++ fixture for nth-tail.
long long nth_tail(std::vector<long long> marks, std::vector<long long> nxt, long long n) {
    if (nxt.empty()) { return -1; }
    long long lead = 0, trail = 0;
    for (long long k = 0; k < n; k += 1) {
        if (lead == -1) { return -1; }
        lead = nxt[(size_t) lead];
    }
    if (lead == -1) { return -1; }
    while (nxt[(size_t) lead] != -1) { lead = nxt[(size_t) lead]; trail = nxt[(size_t) trail]; }
    return marks[(size_t) trail];
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(nth_tail(
        from_json<std::vector<long long>>(args[0]),
        from_json<std::vector<long long>>(args[1]),
        from_json<long long>(args[2])));
}
