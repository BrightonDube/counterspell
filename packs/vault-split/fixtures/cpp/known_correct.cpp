// Test-only known-correct C++ fixture for vault-split.
long long split_diff(std::vector<long long> marks) {
    long long total = 0;
    for (long long m : marks) {
        total += m;
    }
    long long half = total / 2;
    std::vector<char> d(half + 1, 0);
    d[0] = 1;
    for (long long m : marks) {
        for (long long i = half; i >= m; i -= 1) {
            if (d[i - m]) {
                d[i] = 1;
            }
        }
    }
    long long best = 0;
    for (long long i = half; i >= 0; i -= 1) {
        if (d[i]) {
            best = i;
            break;
        }
    }
    return total - 2 * best;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(split_diff(from_json<std::vector<long long>>(args[0])));
}
