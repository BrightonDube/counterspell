// Test-only known-correct C++ fixture for corner-turns.
long long corner_turns(std::vector<std::vector<long long>> points) {
    long long count = 0;
    for (size_t i = 1; i + 1 < points.size(); i += 1) {
        long long ux = points[i][0] - points[i - 1][0];
        long long uy = points[i][1] - points[i - 1][1];
        long long vx = points[i + 1][0] - points[i][0];
        long long vy = points[i + 1][1] - points[i][1];
        if (ux * vy - uy * vx > 0) {
            count += 1;
        }
    }
    return count;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(corner_turns(from_json<std::vector<std::vector<long long>>>(args[0])));
}
