// Test-only known-correct C++ fixture for split-ward.
long long can_segment(std::string words, std::string s) {
    std::set<std::string> ws;
    std::string cur;
    std::stringstream stream(words);
    while (std::getline(stream, cur, ',')) {
        if (!cur.empty()) {
            ws.insert(cur);
        }
    }
    std::vector<bool> ok(s.size() + 1, false);
    ok[0] = true;
    for (size_t i = 0; i <= s.size(); i += 1) {
        if (!ok[i]) {
            continue;
        }
        for (const auto& w : ws) {
            if (s.compare(i, w.size(), w) == 0) {
                ok[i + w.size()] = true;
            }
        }
    }
    return ok[s.size()] ? 1 : 0;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(can_segment(from_json<std::string>(args[0]), from_json<std::string>(args[1])));
}
