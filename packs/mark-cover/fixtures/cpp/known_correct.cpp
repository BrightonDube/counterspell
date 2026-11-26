// Test-only known-correct C++ fixture for mark-cover.
long long cover_length(std::vector<std::vector<long long>> spans) {
    std::vector<bool> cov(101, false);
    for (auto& s : spans) {
        for (long long i = s[0]; i <= s[1]; i += 1) {
            cov[i] = true;
        }
    }
    return std::count(cov.begin(), cov.end(), true);
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(cover_length(from_json<std::vector<std::vector<long long>>>(args[0])));
}
