// Test-only known-correct C++ fixture for lone-nodes.
long long lone_count(std::vector<std::vector<long long>> edges, long long n) {
    if (n <= 0) {
        return 0;
    }
    std::vector<char> touched(n, 0);
    for (auto& e : edges) {
        long long a = e[0];
        long long b = e[1];
        if (a >= 0 && a < n) {
            touched[a] = 1;
        }
        if (b >= 0 && b < n) {
            touched[b] = 1;
        }
    }
    long long count = 0;
    for (char t : touched) {
        if (!t) {
            count += 1;
        }
    }
    return count;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(lone_count(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<long long>(args[1])));
}
