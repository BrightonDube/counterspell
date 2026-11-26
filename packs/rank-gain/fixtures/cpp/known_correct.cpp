// Test-only known-correct C++ fixture for rank-gain.
long long rank_gain(std::vector<long long> rels, long long k) {
    long long total = 0;
    for (long long i = 0; i < k && i < (long long)rels.size(); i++) total += rels[i] * (k - i);
    return total;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(rank_gain(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
