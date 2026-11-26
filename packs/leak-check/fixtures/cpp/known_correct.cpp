// Test-only known-correct C++ fixture for leak-check.
long long leak_check(std::vector<long long> train, std::vector<long long> test) {
    std::set<long long> tr(train.begin(), train.end()), both;
    for (long long x : test) if (tr.count(x)) both.insert(x);
    return (long long)both.size();
}


JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(leak_check(from_json<std::vector<long long>>(args[0]), from_json<std::vector<long long>>(args[1])));
}
