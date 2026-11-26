// Test-only known-correct C++ fixture for sigil-board.
static long long f(long long n, long long r, std::vector<long long>& cols) {
    if (r == n) {
        return 1;
    }
    long long total = 0;
    for (long long c = 0; c < n; c += 1) {
        if (std::find(cols.begin(), cols.end(), c) != cols.end()) {
            continue;
        }
        bool clash = false;
        for (size_t rr = 0; rr < cols.size(); rr += 1) {
            if (std::llabs(r - (long long) rr) == std::llabs(c - cols[rr])) {
                clash = true;
                break;
            }
        }
        if (!clash) {
            cols.push_back(c);
            total += f(n, r + 1, cols);
            cols.pop_back();
        }
    }
    return total;
}

long long sigil_count(long long n) {
    std::vector<long long> cols;
    return f(n, 0, cols);
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(sigil_count(from_json<long long>(args[0])));
}
