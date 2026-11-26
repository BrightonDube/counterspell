// Test-only known-correct C++ fixture for chain-length.
long long chain_length(std::vector<long long> nxt) {
    long long i = nxt.empty() ? -1 : 0, count = 0;
    while (i != -1) { count += 1; i = nxt[(size_t) i]; }
    return count;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(chain_length(from_json<std::vector<long long>>(args[0])));
}
