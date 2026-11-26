// Test-only known-correct C++ fixture for digit-rite.
long long digit_rite(long long n) {
    long long orig = n, count = 0;
    while (n > 0) {
        long long d = n % 10;
        if (d != 0 && orig % d == 0) {
            count += 1;
        }
        n /= 10;
    }
    return count;
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(digit_rite(from_json<long long>(args[0])));
}
