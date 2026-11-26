// Test-only known-correct C++ fixture for tally-frames.
long long tally_frames(std::vector<std::vector<long long>> m) {
    long long total = 0;
    for (size_t i = 0; i < m.size(); i++) if (i < m[i].size()) total += m[i][i];
    return total;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(tally_frames(from_json<std::vector<std::vector<long long>>>(args[0])));
}
