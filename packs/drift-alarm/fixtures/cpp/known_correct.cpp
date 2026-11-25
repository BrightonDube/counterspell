// Test-only known-correct C++ fixture for drift-alarm.
long long drift_alarms(std::vector<long long> readings, long long width) {
    long long alarms = 0;
    for (std::size_t i = (std::size_t) width; i < readings.size(); i += 1) {
        long long peak = readings[i - (std::size_t) width];
        for (std::size_t j = i - (std::size_t) width + 1; j < i; j += 1) {
            if (readings[j] > peak) {
                peak = readings[j];
            }
        }
        if (readings[i] > peak) {
            alarms += 1;
        }
    }
    return alarms;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(drift_alarms(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1])));
}
