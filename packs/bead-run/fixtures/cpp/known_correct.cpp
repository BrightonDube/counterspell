// Test-only known-correct C++ fixture for bead-run.
long long longest_bead(std::string strand) {
    long long best = 0;
    long long run = 0;
    char previous = 0;
    for (char bead : strand) {
        run = bead == previous ? run + 1 : 1;
        if (run > best) {
            best = run;
        }
        previous = bead;
    }
    return best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(longest_bead(from_json<std::string>(args[0])));
}
