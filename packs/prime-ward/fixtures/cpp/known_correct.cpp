// Test-only known-correct C++ fixture for prime-ward.
long long prime_ward(long long n) {
    if (n < 2) {
        return 0;
    }
    std::vector<bool> sieve(n + 1, true);
    sieve[0] = sieve[1] = false;
    for (long long p = 2; p * p <= n; p += 1) {
        if (sieve[p]) {
            for (long long k = p * p; k <= n; k += p) {
                sieve[k] = false;
            }
        }
    }
    long long count = 0;
    for (bool s : sieve) {
        count += s;
    }
    return count;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(prime_ward(from_json<long long>(args[0])));
}
