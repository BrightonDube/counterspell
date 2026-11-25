// Test-only known-correct C++ fixture for warm-window.
long long warmest_window(std::vector<long long> readings, long long width) {
    if (readings.size() < (std::size_t) width) {
        return 0;
    }
    long long total = 0;
    for (long long i = 0; i < width; i += 1) {
        total += readings[i];
    }
    long long best = total;
    for (std::size_t i = (std::size_t) width; i < readings.size(); i += 1) {
        total += readings[i] - readings[i - (std::size_t) width];
        if (total > best) {
            best = total;
        }
    }
    return best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(warmest_window(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1])));
}
