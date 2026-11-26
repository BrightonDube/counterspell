// Test-only known-correct C++ fixture for lru-rite.
long long lru_rite(std::vector<long long> pages, long long f) {
    std::deque<long long> folio;
    long long faults = 0;
    for (long long p : pages) {
        auto at = std::find(folio.begin(), folio.end(), p);
        if (at != folio.end()) {
            folio.erase(at);
            folio.push_back(p);
        } else {
            faults += 1;
            if ((long long)folio.size() == f) folio.pop_front();
            folio.push_back(p);
        }
    }
    return faults;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(lru_rite(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
