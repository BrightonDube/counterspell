// Test-only known-correct C++ fixture for gale-peaks.
std::vector<long long> gale_peaks(std::vector<long long> readings, long long width) {
    std::vector<long long> peaks;
    std::deque<std::size_t> candidates;
    for (std::size_t i = 0; i < readings.size(); i += 1) {
        while (!candidates.empty() && readings[candidates.back()] <= readings[i]) {
            candidates.pop_back();
        }
        candidates.push_back(i);
        if (candidates.front() + (std::size_t) width <= i) {
            candidates.pop_front();
        }
        if (i + 1 >= (std::size_t) width) {
            peaks.push_back(readings[candidates.front()]);
        }
    }
    return peaks;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(gale_peaks(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1])));
}
