// Test-only known-correct C++ fixture for null-marks.
long long null_marks(std::vector<std::vector<long long>> rows) {
    long long count = 0;
    for (auto& r : rows) {
        if (r[1] == -1) {
            count += 1;
        }
    }
    return count;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(null_marks(from_json<std::vector<std::vector<long long>>>(args[0])));
}
