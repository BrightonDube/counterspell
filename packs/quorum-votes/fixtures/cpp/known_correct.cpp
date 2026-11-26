// Test-only known-correct C++ fixture for quorum-votes.
long long quorum_reached(long long n, long long votes) {
    return votes >= n / 2 + 1 ? 1 : 0;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(quorum_reached(from_json<long long>(args[0]), from_json<long long>(args[1])));
}
