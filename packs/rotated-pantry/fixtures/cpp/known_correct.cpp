// Test-only known-correct C++ fixture for rotated-pantry.
long long find_in_rotated(std::vector<long long> stores, long long target) {
    long long lo = 0, hi = (long long) stores.size() - 1;
    while (lo <= hi) {
        long long mid = (lo + hi) / 2;
        if (stores[(size_t) mid] == target) return mid;
        if (stores[(size_t) lo] <= stores[(size_t) mid]) {
            if (stores[(size_t) lo] <= target && target < stores[(size_t) mid]) { hi = mid - 1; } else { lo = mid + 1; }
        } else {
            if (stores[(size_t) mid] < target && target <= stores[(size_t) hi]) { lo = mid + 1; } else { hi = mid - 1; }
        }
    }
    return -1;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(find_in_rotated(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1])));
}
