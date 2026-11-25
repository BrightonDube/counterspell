// Test-only known-correct C++ fixture for merge-glow.
std::vector<long long> merge_glow(std::vector<long long> am, std::vector<long long> an,
                                  std::vector<long long> bm, std::vector<long long> bn) {
    std::vector<long long> out;
    long long i = an.empty() ? -1 : 0, j = bn.empty() ? -1 : 0;
    while (i != -1 && j != -1) {
        if (am[(size_t) i] <= bm[(size_t) j]) { out.push_back(am[(size_t) i]); i = an[(size_t) i]; }
        else { out.push_back(bm[(size_t) j]); j = bn[(size_t) j]; }
    }
    for (; i != -1; i = an[(size_t) i]) { out.push_back(am[(size_t) i]); }
    for (; j != -1; j = bn[(size_t) j]) { out.push_back(bm[(size_t) j]); }
    return out;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(merge_glow(
        from_json<std::vector<long long>>(args[0]),
        from_json<std::vector<long long>>(args[1]),
        from_json<std::vector<long long>>(args[2]),
        from_json<std::vector<long long>>(args[3])));
}
