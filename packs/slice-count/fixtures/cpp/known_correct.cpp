// Test-only known-correct C++ fixture for slice-count.
long long slice_count(std::vector<long long> bursts, long long q) {
    long long total = 0;
    for (long long b : bursts) {
        total += (b + q - 1) / q;
    }
    return total;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(slice_count(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
