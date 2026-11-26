// Test-only known-correct C++ fixture for lock-order.
long long lock_order(std::vector<long long> wards, std::vector<long long> lens) {
    std::set<std::pair<long long, long long>> seen;
    size_t at = 0;
    for (long long count : lens) {
        std::vector<long long> row(wards.begin() + at, wards.begin() + at + count);
        at += count;
        for (size_t i = 0; i < row.size(); i++) {
            for (size_t j = i + 1; j < row.size(); j++) {
                long long a = row[i], b = row[j];
                if (a != b && seen.count({b, a})) return 0;
                seen.insert({a, b});
            }
        }
    }
    return 1;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(lock_order(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1])));
}
