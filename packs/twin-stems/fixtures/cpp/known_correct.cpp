// Test-only known-correct C++ fixture for twin-stems.
long long prefix_pairs(std::string words) {
    std::set<std::string> ws;
    std::string cur;
    std::stringstream stream(words);
    while (std::getline(stream, cur, ',')) {
        if (!cur.empty()) {
            ws.insert(cur);
        }
    }
    std::vector<std::string> list(ws.begin(), ws.end());
    long long ans = 0;
    for (size_t i = 0; i < list.size(); i += 1) {
        for (size_t j = i + 1; j < list.size(); j += 1) {
            if (list[i] != list[j] && list[j].rfind(list[i], 0) == 0) {
                ans += 1;
            }
        }
    }
    return ans;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(prefix_pairs(from_json<std::string>(args[0])));
}
