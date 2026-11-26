// Test-only known-correct C++ fixture for climb-rite.
long long climb_ways(long long steps) {
    if (steps <= 0) {
        return 1;
    }
    long long a = 1;
    long long b = 1;
    for (long long i = 0; i < steps - 1; i += 1) {
        long long t = a + b;
        a = b;
        b = t;
    }
    return b;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(climb_ways(from_json<long long>(args[0])));
}
