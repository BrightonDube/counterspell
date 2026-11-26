// Test-only known-correct C++ fixture for first-free.
long long first_free(std::vector<std::vector<long long>> spans) {
    std::vector<bool> cov(101, false);
    for (auto& s : spans) {
        for (long long i = s[0]; i <= s[1]; i += 1) {
            cov[i] = true;
        }
    }
    long long i = 0;
    while (i <= 100 && cov[i]) {
        i += 1;
    }
    return i;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(first_free(from_json<std::vector<std::vector<long long>>>(args[0])));
}
