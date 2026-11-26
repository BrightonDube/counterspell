// Test-only known-correct C++ fixture for prefix-tally.
long long prefix_count(std::string words, std::string prefix) {
    long long count = 0;
    std::string cur;
    std::stringstream stream(words);
    while (std::getline(stream, cur, ',')) {
        if (!cur.empty() && cur.rfind(prefix, 0) == 0) {
            count += 1;
        }
    }
    return count;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(prefix_count(from_json<std::string>(args[0]), from_json<std::string>(args[1])));
}
