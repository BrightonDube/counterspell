// Test-only known-correct C++ fixture for page-turn.
long long page_turn(std::vector<long long> pages, long long f) {
    std::deque<long long> folio;
    long long faults = 0;
    for (long long p : pages) {
        if (std::find(folio.begin(), folio.end(), p) == folio.end()) {
            faults += 1;
            if ((long long)folio.size() == f) folio.pop_front();
            folio.push_back(p);
        }
    }
    return faults;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(page_turn(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
