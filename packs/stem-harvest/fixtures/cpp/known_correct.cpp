// Test-only known-correct C++ fixture for stem-harvest.
long long stem_depth_sum(std::string words) {
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
        size_t best = 0;
        for (const auto& o : ws) {
            if (o != w && w.rfind(o, 0) == 0 && o.size() > best) {
                best = o.size();
            }
        }
        ans += (long long) best;
    }
    return ans;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(stem_depth_sum(from_json<std::string>(args[0])));
}
