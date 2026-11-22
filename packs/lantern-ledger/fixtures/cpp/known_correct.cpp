// Test-only known-correct C++ fixture for lantern-ledger.
long long lantern_ledger(std::vector<long long> changes, long long start, long long level) {
    long long meter = start;
    long long marks = 0;
    for (long long change : changes) {
        meter += change;
        if (meter == level) {
            marks += 1;
        }
    }
    return marks;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(lantern_ledger(from_json<std::vector<long long>>(args[0]),
                                  from_json<long long>(args[1]),
                                  from_json<long long>(args[2])));
}
