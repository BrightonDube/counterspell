// Test-only known-correct C++ fixture for deepest-lex.
long long longest_word(std::string words) {
    long long best = 0;
    std::string cur;
    std::stringstream stream(words);
    while (std::getline(stream, cur, ',')) {
        if (!cur.empty() && (long long) cur.size() > best) {
            best = (long long) cur.size();
        }
    }
    return best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(longest_word(from_json<std::string>(args[0])));
}
