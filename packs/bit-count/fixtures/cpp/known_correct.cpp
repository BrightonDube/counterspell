// Test-only known-correct C++ fixture for bit-count.
long long bit_count(long long n) {
    long long count = 0;
    while (n > 0) {
        count += n & 1;
        n >>= 1;
    }
    return count;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(bit_count(from_json<long long>(args[0])));
}
