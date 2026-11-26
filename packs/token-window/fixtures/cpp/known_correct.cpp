// Test-only known-correct C++ fixture for token-window.
long long token_window(std::vector<long long> tokens, long long w, long long mark) {
    long long width = std::min(w, (long long)tokens.size());
    if (width <= 0) return 0;
    long long cur = 0;
    for (long long i = 0; i < width; i++) if (tokens[i] == mark) cur++;
    long long best = cur;
    for (long long i = width; i < (long long)tokens.size(); i++) {
        cur += (tokens[i] == mark ? 1 : 0) - (tokens[i - width] == mark ? 1 : 0);
        if (cur > best) best = cur;
    }
    return best;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(token_window(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1]), from_json<long long>(args[2])));
}
