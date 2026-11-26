// Test-only known-correct C++ fixture for alloc-rite.
long long alloc_rite(std::vector<long long> blocks, std::vector<long long> reqs) {
    long long granted = 0;
    for (long long r : reqs) {
        for (size_t i = 0; i < blocks.size(); i++) {
            if (blocks[i] >= r) { blocks[i] -= r; granted += 1; break; }
        }
    }
    return granted;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(alloc_rite(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1])));
}
