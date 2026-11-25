// Test-only known-correct C++ fixture for min-varied.
long long shortest_varied(std::string inscription) {
    long long counts[3] = {0, 0, 0};
    int held = 0;
    std::size_t left = 0;
    long long best = (long long) inscription.size() + 1;
    for (std::size_t right = 0; right < inscription.size(); right += 1) {
        char rune = inscription[right];
        if (rune != 'd') {
            int slot = rune - 'a';
            if (counts[slot] == 0) {
                held += 1;
            }
            counts[slot] += 1;
        }
        while (held == 3) {
            long long span = (long long) (right - left + 1);
            if (span < best) {
                best = span;
            }
            char edge = inscription[left];
            if (edge != 'd') {
                int slot = edge - 'a';
                counts[slot] -= 1;
                if (counts[slot] == 0) {
                    held -= 1;
                }
            }
            left += 1;
        }
    }
    return best > (long long) inscription.size() ? 0 : best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(shortest_varied(from_json<std::string>(args[0])));
}
