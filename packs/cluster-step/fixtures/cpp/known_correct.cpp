// Test-only known-correct C++ fixture for cluster-step.
long long cluster_step(std::vector<long long> points, std::vector<long long> centroids, std::vector<long long> labels) {
    if (centroids.empty()) return (long long)points.size();
    long long moved = 0;
    for (size_t i = 0; i < points.size(); i++) {
        size_t best = 0;
        for (size_t j = 1; j < centroids.size(); j++) {
            if (llabs(points[i] - centroids[j]) < llabs(points[i] - centroids[best])) best = j;
        }
        if (i >= labels.size() || labels[i] != (long long)best) moved++;
    }
    return moved;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(cluster_step(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1]), from_json<std::vector<long long>>(args[2])));
}
