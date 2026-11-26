// Test-only known-correct C++ fixture for collinear-marks.
long long collinear_marks(std::vector<std::vector<long long>> points) {
    if (points.size() < 2 || (points[0][0] == points[1][0] && points[0][1] == points[1][1])) {
        return -1;
    }
    long long x0 = points[0][0], y0 = points[0][1];
    long long dx = points[1][0] - x0, dy = points[1][1] - y0;
    long long count = 0;
    for (auto& p : points) {
        if ((p[0] - x0) * dy == (p[1] - y0) * dx) {
            count += 1;
        }
    }
    return count;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(collinear_marks(from_json<std::vector<std::vector<long long>>>(args[0])));
}
