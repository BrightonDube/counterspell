// Test-only known-correct C++ fixture for top-marks.
long long top_marks(std::vector<long long> scores, long long k) {
    std::sort(scores.rbegin(), scores.rend());
    long long total = 0;
    for (long long i = 0; i < k && i < (long long)scores.size(); i++) total += scores[i];
    return total;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(top_marks(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
