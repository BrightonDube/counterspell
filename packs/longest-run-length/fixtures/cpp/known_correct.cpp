// Test-only known-correct C++ fixture for longest-run-length.
long long longest_run(std::string text) {
    long long best = 0;
    long long streak = 0;
    char last = 0;
    for (std::size_t i = 0; i < text.size(); i++) {
        streak = (i > 0 && text[i] == last) ? streak + 1 : 1;
        last = text[i];
        if (streak > best) {
            best = streak;
        }
    }
    return best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(longest_run(from_json<std::string>(args[0])));
}
