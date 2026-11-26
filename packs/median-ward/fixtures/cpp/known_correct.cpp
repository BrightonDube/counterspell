// Test-only known-correct C++ fixture for median-ward.
long long median_x2(std::vector<long long> marks) {
    std::sort(marks.begin(), marks.end());
    size_t n = marks.size();
    if (n == 0) {
        return 0;
    }
    if (n % 2 == 1) {
        return 2 * marks[n / 2];
    }
    return marks[n / 2 - 1] + marks[n / 2];
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(median_x2(from_json<std::vector<long long>>(args[0])));
}
