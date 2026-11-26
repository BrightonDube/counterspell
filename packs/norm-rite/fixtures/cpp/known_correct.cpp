// Test-only known-correct C++ fixture for norm-rite.
long long norm_rite(std::vector<long long> v) {
    long long s = 0;
    for (long long x : v) s += x;
    if (s <= 0) return 0;
    long long shares = 0;
    for (long long x : v) shares += (x * 100) / s;
    return 100 - shares;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(norm_rite(from_json<std::vector<long long>>(args[0])));
}
