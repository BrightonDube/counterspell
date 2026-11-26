// Test-only known-correct C++ fixture for bit-isolate.
long long bit_isolate(long long n) {
    return n & (-n);
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(bit_isolate(from_json<long long>(args[0])));
}
