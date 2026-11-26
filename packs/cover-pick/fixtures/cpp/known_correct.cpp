// Test-only known-correct C++ fixture for cover-pick.
long long cover_pick(long long n, std::vector<long long> masks) {
    long long full = (1LL << n) - 1, covered = 0, picks = 0;
    while (covered != full) {
        long long gain = 0; long long best = -1;
        for (size_t i = 0; i < masks.size(); i++) {
            long long g = __builtin_popcountll(masks[i] & full & ~covered);
            if (g > gain) { gain = g; best = (long long)i; }
        }
        if (best < 0) return picks;
        covered |= masks[best] & full;
        picks++;
    }
    return picks;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(cover_pick(from_json<long long>(args[0]), from_json<std::vector<long long>>(args[1])));
}
