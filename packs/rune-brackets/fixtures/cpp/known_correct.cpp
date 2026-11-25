// Test-only known-correct C++ fixture for rune-brackets.
long long nested_runes(std::string inscription) {
    std::vector<char> openers;
    for (char rune : inscription) {
        if (rune == '(' || rune == '[' || rune == '{') {
            openers.push_back(rune);
        } else {
            if (openers.empty()) {
                return 0;
            }
            char open = openers.back();
            bool matches = (open == '(' && rune == ')')
                || (open == '[' && rune == ']')
                || (open == '{' && rune == '}');
            if (!matches) {
                return 0;
            }
            openers.pop_back();
        }
    }
    return openers.empty() ? 1 : 0;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(nested_runes(from_json<std::string>(args[0])));
}
