// Test-only known-correct C++ fixture for mod-toll.
long long mod_toll(std::vector<long long> values, long long m) {
    long long r = 0;
    for (long long v : values) {
        r += v;
    }
    return ((r % m) + m) % m;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(mod_toll(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
