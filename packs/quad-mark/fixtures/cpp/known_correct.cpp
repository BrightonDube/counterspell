// Test-only known-correct C++ fixture for quad-mark.
long long quad_mark(long long x, long long y) {
    if (x == 0 || y == 0) {
        return 0;
    }
    if (y > 0) {
        return x > 0 ? 1 : 2;
    }
    return x > 0 ? 4 : 3;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(quad_mark(from_json<long long>(args[0]), from_json<long long>(args[1])));
}
