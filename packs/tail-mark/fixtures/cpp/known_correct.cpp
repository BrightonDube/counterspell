// Test-only known-correct C++ fixture for tail-mark.
long long tail_mark(std::vector<long long> marks, std::vector<long long> nxt) {
    long long i = nxt.empty() ? -1 : 0, last = -1;
    while (i != -1) { last = i; i = nxt[(size_t) i]; }
    return last == -1 ? -1 : marks[(size_t) last];
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(tail_mark(
        from_json<std::vector<long long>>(args[0]),
        from_json<std::vector<long long>>(args[1])));
}
