// Test-only known-correct C++ fixture for deadline-dial.
long long dial_count(std::vector<long long> slots) {
    std::vector<std::pair<long long, long long>> jobs;
    for (size_t i = 0; i + 1 < slots.size(); i += 2) {
        jobs.emplace_back(slots[i], slots[i + 1]);
    }
    std::sort(jobs.begin(), jobs.end());
    std::priority_queue<long long> held;
    long long time = 0;
    for (const auto& job : jobs) {
        time += job.second;
        held.push(job.second);
        if (time > job.first) {
            time -= held.top();
            held.pop();
        }
    }
    return (long long) held.size();
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(dial_count(from_json<std::vector<long long>>(args[0])));
}
