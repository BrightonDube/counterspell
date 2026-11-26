// Test-only known-correct C++ fixture for calm-gaps.
long long calm_haul(std::vector<long long> marks) {
    long long take = 0;
    long long skip = 0;
    for (long long m : marks) {
        long long t = skip + m;
        skip = std::max(skip, take);
        take = t;
    }
    return std::max(take, skip);
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(calm_haul(from_json<std::vector<long long>>(args[0])));
}
