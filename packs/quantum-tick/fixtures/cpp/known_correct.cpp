// Test-only known-correct C++ fixture for quantum-tick.
long long quantum_tick(std::vector<std::vector<long long>> jobs, long long q) {
    int n = (int)jobs.size();
    std::vector<int> order(n);
    for (int i = 0; i < n; i++) order[i] = i;
    std::sort(order.begin(), order.end(), [&](int x, int y) {
        return jobs[x][0] != jobs[y][0] ? jobs[x][0] < jobs[y][0] : x < y;
    });
    std::vector<long long> rem(n);
    for (int i = 0; i < n; i++) rem[i] = jobs[i][1];
    std::deque<int> ready;
    int i = 0, done = 0;
    long long t = 0, total = 0;
    while (done < n) {
        while (i < n && jobs[order[i]][0] <= t) { ready.push_back(order[i]); i += 1; }
        if (ready.empty()) { t = jobs[order[i]][0]; continue; }
        int j = ready.front(); ready.pop_front();
        long long run = std::min(q, rem[j]);
        t += run;
        rem[j] -= run;
        while (i < n && jobs[order[i]][0] <= t) { ready.push_back(order[i]); i += 1; }
        if (rem[j] > 0) ready.push_back(j);
        else { total += t; done += 1; }
    }
    return total;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(quantum_tick(from_json<std::vector<std::vector<long long>>>(args[0]), from_json<long long>(args[1])));
}
