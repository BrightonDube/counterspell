// Test-only known-correct C++ fixture for deepest-ember.
long long deepest_ember(std::string inscription) {
    long long depth = 0;
    long long deepest = 0;
    for (char rune : inscription) {
        if (rune == '(') {
            depth += 1;
            if (depth > deepest) {
                deepest = depth;
            }
        } else if (depth > 0) {
            depth -= 1;
        }
    }
    return deepest;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(deepest_ember(from_json<std::string>(args[0])));
}
