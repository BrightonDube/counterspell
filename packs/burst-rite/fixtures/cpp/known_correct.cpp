// Test-only known-correct C++ fixture for burst-rite.
long long burst_rite(std::vector<std::vector<long long>> jobs) {
    std::vector<int> order(jobs.size());
    for (int i = 0; i < (int)jobs.size(); i++) order[i] = i;
    std::sort(order.begin(), order.end(), [&](int x, int y) {
        return jobs[x][0] != jobs[y][0] ? jobs[x][0] < jobs[y][0] : x < y;
    });
    long long clock = 0, total = 0;
    for (int i : order) {
        long long a = jobs[i][0], b = jobs[i][1];
        long long start = std::max(clock, a);
        total += start - a;
        clock = start + b;
    }
    return total;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(burst_rite(from_json<std::vector<std::vector<long long>>>(args[0])));
}
