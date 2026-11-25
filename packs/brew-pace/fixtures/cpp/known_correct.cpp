// Test-only known-correct C++ fixture for brew-pace.
long long min_pace(std::vector<long long> brews, long long hours) {
    long long lo = 1, hi = 1;
    for (long long t : brews) { if (t > hi) { hi = t; } }
    while (lo < hi) {
        long long mid = (lo + hi) / 2;
        long long need = 0;
        for (long long t : brews) { need += (t + mid - 1) / mid; }
        if (need <= hours) { hi = mid; } else { lo = mid + 1; }
    }
    return lo;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(min_pace(
        from_json<std::vector<long long>>(args[0]),
        from_json<long long>(args[1])));
}
