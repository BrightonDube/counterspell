// Test-only known-correct C++ fixture for sorted-embers (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
std::vector<long long> ordered_squares(std::vector<long long> readings) {
    std::vector<long long> answer(readings.size());
    std::size_t left = 0, right = readings.size() - 1;
    for (std::size_t write = readings.size(); write > 0; write -= 1) {
        long long lo = readings[left] * readings[left];
        long long hi = readings[right] * readings[right];
        if (hi >= lo) {
            answer[write - 1] = hi;
            right -= 1;
        } else {
            answer[write - 1] = lo;
            left += 1;
        }
    }
    return answer;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(ordered_squares(from_json<std::vector<long long>>(args[0])));
}
