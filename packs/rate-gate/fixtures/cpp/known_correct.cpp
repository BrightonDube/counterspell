// Test-only known-correct C++ fixture for rate-gate.
long long rate_gate(std::vector<std::vector<long long>> events, long long capacity, long long refill) {
    long long bucket = capacity, last = 0, passed = 0;
    for (auto& e : events) {
        bucket = std::min(capacity, bucket + (e[0] - last) * refill);
        last = e[0];
        if (e[1] <= bucket) { bucket -= e[1]; passed++; }
    }
    return passed;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(rate_gate(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<long long>(args[1]), from_json<long long>(args[2])));
}
