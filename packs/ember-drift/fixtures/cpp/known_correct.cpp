// Test-only known-correct C++ fixture for ember-drift (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
std::vector<long long> drift_dark(std::vector<long long> marks) {
    std::vector<long long> drifted(marks.size(), 0);
    std::size_t write = 0;
    for (long long mark : marks) {
        if (mark != 0) {
            drifted[write] = mark;
            write += 1;
        }
    }
    return drifted;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(drift_dark(from_json<std::vector<long long>>(args[0])));
}
