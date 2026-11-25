// Test-only known-correct C++ fixture for loop-entry.
long long loop_entry(std::vector<long long> nxt) {
    std::set<long long> seen;
    long long i = nxt.empty() ? -1 : 0;
    while (i != -1 && !seen.count(i)) { seen.insert(i); i = nxt[(size_t) i]; }
    return i;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(loop_entry(from_json<std::vector<long long>>(args[0])));
}
