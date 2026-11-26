// Test-only known-correct C++ fixture for word-ward.
long long has_word(std::string words, std::string word) {
    if (word.empty()) {
        return 0;
    }
    std::string cur;
    std::stringstream stream(words);
    while (std::getline(stream, cur, ',')) {
        if (cur == word) {
            return 1;
        }
    }
    return 0;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(has_word(from_json<std::string>(args[0]), from_json<std::string>(args[1])));
}
