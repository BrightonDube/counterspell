// Test-only known-correct C++ fixture for candle-decode.
long long decode_ways(std::string glyphs) {
    long long n = (long long) glyphs.size();
    if (n == 0) {
        return 1;
    }
    std::vector<long long> d(n + 1, 0);
    d[0] = 1;
    for (long long i = 1; i <= n; i += 1) {
        if (glyphs[i - 1] != '0') {
            d[i] += d[i - 1];
        }
        if (i >= 2) {
            int pair = std::stoi(glyphs.substr(i - 2, 2));
            if (pair >= 10 && pair <= 26) {
                d[i] += d[i - 2];
            }
        }
    }
    return d[n];
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(decode_ways(from_json<std::string>(args[0])));
}
