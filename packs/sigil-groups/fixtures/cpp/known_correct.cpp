// Test-only known-correct C++ fixture for sigil-groups (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
long long largest_sigil(std::vector<long long> markings) {
    long long groups[28] = {};
    for (long long marking : markings) {
        long long sigil = 0;
        for (long long rest = marking; rest > 0; rest /= 10) sigil += rest % 10;
        groups[sigil] += 1;
    }
    long long best = 0;
    for (long long tally : groups) best = std::max(best, tally);
    return best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(largest_sigil(from_json<std::vector<long long>>(args[0])));
}
