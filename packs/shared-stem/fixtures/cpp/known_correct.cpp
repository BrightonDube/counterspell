// Test-only known-correct C++ fixture for shared-stem.
long long common_stem(std::string words) {
    std::string cur;
    std::stringstream stream(words);
    std::string stem;
    bool have = false;
    while (std::getline(stream, cur, ',')) {
        if (cur.empty()) {
            continue;
        }
        if (!have) {
            stem = cur;
            have = true;
            continue;
        }
        while (cur.rfind(stem, 0) != 0) {
            stem.pop_back();
        }
    }
    return (long long) stem.size();
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(common_stem(from_json<std::string>(args[0])));
}
