// Test-only known-correct C++ fixture for wrap-rite.
long long wrap_rite(std::vector<long long> marks, long long b) {
    std::deque<long long> ring;
    long long evictions = 0;
    for (long long m : marks) {
        if (std::find(ring.begin(), ring.end(), m) == ring.end()) {
            if ((long long)ring.size() == b) { ring.pop_front(); evictions += 1; }
            ring.push_back(m);
        }
    }
    return evictions;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(wrap_rite(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
