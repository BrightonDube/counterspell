// Test-only known-correct C++ fixture for ember-words.
static std::vector<std::string> split_embers(const std::string& lexicon) {
    std::vector<std::string> out;
    std::string cur;
    for (char c : lexicon) {
        if (c == ',') {
            if (!cur.empty()) out.push_back(cur);
            cur.clear();
        } else {
            cur += c;
        }
    }
    if (!cur.empty()) out.push_back(cur);
    return out;
}

long long word_break(std::string text, std::string lexicon) {
    auto words = split_embers(lexicon);
    std::vector<long long> d(text.size() + 1, 0);
    d[0] = 1;
    for (size_t i = 1; i <= text.size(); i += 1) {
        for (auto& w : words) {
            if (i >= w.size() && d[i - w.size()] && text.compare(i - w.size(), w.size(), w) == 0) {
                d[i] = 1;
                break;
            }
        }
    }
    return d[text.size()];
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(word_break(from_json<std::string>(args[0]), from_json<std::string>(args[1])));
}
