// Test-only known-correct C++ fixture for undo-ritual.
std::string surviving_text(std::string chant) {
    std::string written;
    for (char rune : chant) {
        if (rune == '#') {
            if (!written.empty()) {
                written.pop_back();
            }
        } else {
            written.push_back(rune);
        }
    }
    return written;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(surviving_text(from_json<std::string>(args[0])));
}
