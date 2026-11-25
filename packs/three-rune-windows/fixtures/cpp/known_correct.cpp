// Test-only known-correct C++ fixture for three-rune-windows.
long long count_varied(std::string inscription) {
    std::size_t n = inscription.size();
    long long total = 0;
    for (std::size_t left = 0; left < n; left += 1) {
        bool seen[3] = {false, false, false};
        int missing = 3;
        for (std::size_t right = left; right < n; right += 1) {
            int slot = inscription[right] - 'a';
            if (!seen[slot]) {
                seen[slot] = true;
                missing -= 1;
            }
            if (missing == 0) {
                total += (long long) (n - right);
                break;
            }
        }
    }
    return total;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(count_varied(from_json<std::string>(args[0])));
}
