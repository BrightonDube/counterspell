// Test-only known-correct C++ fixture for candle-compact (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
long long remaining_candles(std::vector<long long> marks, long long cursed) {
    long long kept = 0;
    for (long long mark : marks) {
        if (mark != cursed) {
            kept += 1;
        }
    }
    return kept;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(remaining_candles(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1])));
}
