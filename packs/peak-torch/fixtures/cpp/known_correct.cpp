// Test-only known-correct C++ fixture for peak-torch.
long long peak_torch(std::vector<long long> heights) {
    long long lo = 0, hi = (long long) heights.size() - 1;
    while (lo < hi) {
        long long mid = (lo + hi) / 2;
        if (heights[(size_t) mid] < heights[(size_t) mid + 1]) { lo = mid + 1; } else { hi = mid; }
    }
    return lo;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(peak_torch(from_json<std::vector<long long>>(args[0])));
}
