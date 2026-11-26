// Test-only known-correct C++ fixture for mask-rite.
long long mask_rite(std::vector<long long> weights, long long cap) {
    long long n = (long long) weights.size();
    long long count = 0;
    for (long long mask = 0; mask < (1LL << n); mask += 1) {
        long long total = 0;
        for (long long i = 0; i < n; i += 1) {
            if ((mask >> i) & 1LL) {
                total += weights[i];
            }
        }
        if (total <= cap) {
            count += 1;
        }
    }
    return count;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(mask_rite(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
