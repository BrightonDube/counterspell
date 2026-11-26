// Test-only known-correct C++ fixture for next-rune.
long long next_letters(std::string words, std::string prefix) {
    std::set<char> nxts;
    size_t k = prefix.size();
    std::string cur;
    std::stringstream stream(words);
    while (std::getline(stream, cur, ',')) {
        if (!cur.empty() && cur.rfind(prefix, 0) == 0 && cur.size() > k) {
            nxts.insert(cur[k]);
        }
    }
    return (long long) nxts.size();
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(next_letters(from_json<std::string>(args[0]), from_json<std::string>(args[1])));
}
