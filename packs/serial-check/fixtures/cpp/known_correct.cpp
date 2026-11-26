// Test-only known-correct C++ fixture for serial-check.
long long serial_check(std::vector<long long> events) {
    std::set<long long> done;
    long long prev = -1;
    bool started = false;
    for (long long t : events) {
        if (done.count(t)) {
            return 0;
        }
        if (!started || t != prev) {
            if (started) {
                done.insert(prev);
            }
            prev = t;
            started = true;
        }
    }
    return 1;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(serial_check(from_json<std::vector<long long>>(args[0])));
}
