// Test-only known-correct C++ fixture for mirror-signal.
long long mirror_signal(std::string signal) {
    long long mismatches = 0;
    for (std::size_t left = 0; left < signal.size() / 2; left++) {
        std::size_t right = signal.size() - left - 1;
        if (signal[left] != signal[right]) {
            mismatches += 1;
        }
    }
    return mismatches;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(mirror_signal(from_json<std::string>(args[0])));
}
