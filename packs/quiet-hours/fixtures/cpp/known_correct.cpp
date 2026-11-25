// Test-only known-correct C++ fixture for quiet-hours.
long long quiet_windows(std::vector<long long> readings, long long quota) {
    long long quiet = 0;
    for (std::size_t i = 0; i + 2 < readings.size(); i += 1) {
        if (readings[i] + readings[i + 1] + readings[i + 2] <= quota) {
            quiet += 1;
        }
    }
    return quiet;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(quiet_windows(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1])));
}
