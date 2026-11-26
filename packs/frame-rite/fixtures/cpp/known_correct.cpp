// Test-only known-correct C++ fixture for frame-rite.
long long frame_rite(std::vector<long long> pages, long long f) {
    std::vector<long long> frame;
    long long faults = 0;
    int n = (int)pages.size();
    for (int i = 0; i < n; i++) {
        long long p = pages[i];
        if (std::find(frame.begin(), frame.end(), p) != frame.end()) continue;
        faults += 1;
        if ((long long)frame.size() < f) { frame.push_back(p); continue; }
        int worst = 0, worstNext = -1;
        for (int k = 0; k < (int)frame.size(); k++) {
            int nxt = n + 1;
            for (int j = i + 1; j < n; j++) if (pages[j] == frame[k]) { nxt = j; break; }
            if (nxt > worstNext) { worst = k; worstNext = nxt; }
        }
        frame[worst] = p;
    }
    return faults;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(frame_rite(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
