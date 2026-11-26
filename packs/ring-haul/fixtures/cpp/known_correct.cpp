// Test-only known-correct C++ fixture for ring-haul.
static long long line_run(const std::vector<long long>& ms, int lo, int hi) {
    long long take = 0;
    long long skip = 0;
    for (int i = lo; i < hi; i += 1) {
        long long t = skip + ms[i];
        skip = std::max(skip, take);
        take = t;
    }
    return std::max(take, skip);
}

long long ring_haul(std::vector<long long> marks) {
    if (marks.size() <= 1) {
        return marks.size() == 1 ? marks[0] : 0;
    }
    return std::max(line_run(marks, 1, (int) marks.size()), line_run(marks, 0, (int) marks.size() - 1));
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(ring_haul(from_json<std::vector<long long>>(args[0])));
}
