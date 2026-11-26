// Test-only known-correct C++ fixture for header-check.
long long header_check(std::vector<long long> env) {
    long long v = env[0], s = env[1], q = env[2], l = env[3];
    bool ok = (v == 1 || v == 2) && s >= 0 && s <= 255 && q >= 0 && q <= 65535 && l >= 0 && l <= 1500;
    return ok ? 1 : 0;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(header_check(from_json<std::vector<long long>>(args[0])));
}
