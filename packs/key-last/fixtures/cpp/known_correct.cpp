// Test-only known-correct C++ fixture for key-last.
long long key_last(std::vector<std::vector<long long>> rows, long long k) {
    long long hit = -1;
    for (auto& r : rows) {
        if (r[0] == k) {
            hit = r[1];
        }
    }
    return hit;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(key_last(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<long long>(args[1])));
}
