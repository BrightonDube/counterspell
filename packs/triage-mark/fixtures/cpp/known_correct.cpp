// Test-only known-correct C++ fixture for triage-mark.
long long next_mark(std::vector<long long> prios) {
    long long best = -1;
    for (size_t i = 0; i < prios.size(); i += 1) {
        if (best < 0 || prios[i] < prios[(size_t) best]) {
            best = (long long) i;
        }
    }
    return best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(next_mark(from_json<std::vector<long long>>(args[0])));
}
