// Test-only known-correct C++ fixture for lexicon-shrink.
long long shrunk_count(std::string words) {
    std::set<std::string> ws;
    std::string cur;
    std::stringstream stream(words);
    while (std::getline(stream, cur, ',')) {
        if (!cur.empty()) {
            ws.insert(cur);
        }
    }
    long long ans = 0;
    for (const auto& w : ws) {
        bool absorbed = false;
        for (const auto& o : ws) {
            if (o != w && o.rfind(w, 0) == 0) {
                absorbed = true;
                break;
            }
        }
        if (!absorbed) {
            ans += 1;
        }
    }
    return ans;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(shrunk_count(from_json<std::string>(args[0])));
}
