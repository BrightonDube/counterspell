// Test-only known-correct C++ fixture for longest-rite.
long long longest_nested(std::string inscription) {
    long long best = 0;
    std::vector<long long> stack = {-1};
    for (long long index = 0; index < (long long)inscription.size(); index += 1) {
        if (inscription[index] == '(') {
            stack.push_back(index);
        } else {
            stack.pop_back();
            if (!stack.empty()) {
                long long span = index - stack.back();
                if (span > best) {
                    best = span;
                }
            } else {
                stack.push_back(index);
            }
        }
    }
    return best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(longest_nested(from_json<std::string>(args[0])));
}
