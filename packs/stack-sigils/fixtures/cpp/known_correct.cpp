// Test-only known-correct C++ fixture for stack-sigils.
long long top_sigil(std::vector<long long> ops) {
    std::vector<long long> marks;
    for (long long op : ops) {
        if (op == 0) {
            if (!marks.empty()) {
                marks.pop_back();
            }
        } else {
            marks.push_back(op);
        }
    }
    return marks.empty() ? -1 : marks.back();
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(top_sigil(from_json<std::vector<long long>>(args[0])));
}
