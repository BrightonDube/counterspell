// Test-only known-correct C++ fixture for rune-dial.
std::vector<long long> run_median(std::vector<long long> marks) {
    std::priority_queue<long long> lo;
    std::priority_queue<long long, std::vector<long long>, std::greater<long long>> hi;
    std::vector<long long> out;
    for (long long v : marks) {
        lo.push(v);
        hi.push(lo.top());
        lo.pop();
        if (hi.size() > lo.size()) {
            lo.push(hi.top());
            hi.pop();
        }
        if (lo.size() == hi.size()) {
            out.push_back(lo.top() + hi.top());
        } else {
            out.push_back(2 * lo.top());
        }
    }
    return out;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(run_median(from_json<std::vector<long long>>(args[0])));
}
