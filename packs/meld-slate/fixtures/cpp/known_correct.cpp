// Test-only known-correct C++ fixture for meld-slate.
std::vector<long long> meld(std::vector<long long> a, std::vector<long long> b) {
    std::vector<long long> out;
    size_t i = 0, j = 0;
    while (i < a.size() && j < b.size()) {
        if (a[i] <= b[j]) {
            out.push_back(a[i]); i += 1;
        } else {
            out.push_back(b[j]); j += 1;
        }
    }
    while (i < a.size()) {
        out.push_back(a[i]); i += 1;
    }
    while (j < b.size()) {
        out.push_back(b[j]); j += 1;
    }
    return out;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(meld(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1])));
}
