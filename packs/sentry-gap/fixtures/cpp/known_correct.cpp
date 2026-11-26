// Test-only known-correct C++ fixture for sentry-gap.
static long long f(long long c, long long s, long long gap) {
    if (s == 0) {
        return 1;
    }
    if (c <= 0) {
        return 0;
    }
    return f(c - gap - 1, s - 1, gap) + f(c - 1, s, gap);
}

long long sentry_ways(long long cells, long long sentries, long long gap) {
    return f(cells, sentries, gap);
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(sentry_ways(from_json<long long>(args[0]), from_json<long long>(args[1]), from_json<long long>(args[2])));
}
