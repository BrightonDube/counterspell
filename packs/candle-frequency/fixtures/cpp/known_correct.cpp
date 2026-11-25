// Test-only known-correct C++ fixture for candle-frequency (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
long long top_burn(std::vector<long long> sizes) {
    long long counts[10] = {};
    for (long long size : sizes) counts[size] += 1;
    long long best = 0;
    for (int size = 9; size >= 1; size -= 1) {
        if (counts[size] > 0 && counts[size] >= counts[best]) best = size;
    }
    return best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(top_burn(from_json<std::vector<long long>>(args[0])));
}
