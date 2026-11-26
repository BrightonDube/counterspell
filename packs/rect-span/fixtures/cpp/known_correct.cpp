// Test-only known-correct C++ fixture for rect-span.
long long rect_span(std::vector<long long> a, std::vector<long long> b) {
    long long ax1 = std::min(a[0], a[2]), ax2 = std::max(a[0], a[2]);
    long long ay1 = std::min(a[1], a[3]), ay2 = std::max(a[1], a[3]);
    long long bx1 = std::min(b[0], b[2]), bx2 = std::max(b[0], b[2]);
    long long by1 = std::min(b[1], b[3]), by2 = std::max(b[1], b[3]);
    long long w = std::min(ax2, bx2) - std::max(ax1, bx1);
    long long h = std::min(ay2, by2) - std::max(ay1, by1);
    return std::max(w, 0LL) * std::max(h, 0LL);
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(rect_span(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1])));
}
