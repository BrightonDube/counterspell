// Test-only known-correct C++ fixture for calm-spell.
long long longest_calm(std::string chant) {
    long long best = 0;
    long long current = 0;
    for (char rune : chant) {
        if (rune == '#') {
            current = 0;
        } else {
            current += 1;
            if (current > best) {
                best = current;
            }
        }
    }
    return best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(longest_calm(from_json<std::string>(args[0])));
}
