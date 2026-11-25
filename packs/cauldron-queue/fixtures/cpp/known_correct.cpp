// Test-only known-correct C++ fixture for cauldron-queue.
long long servings(std::vector<long long> ops) {
    std::vector<long long> line(ops.size());
    std::size_t front = 0;
    std::size_t back = 0;
    long long served = 0;
    for (long long op : ops) {
        if (op == 0) {
            if (front < back) {
                front += 1;
                served += 1;
            }
        } else {
            line[back] = op;
            back += 1;
        }
    }
    return served;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(servings(from_json<std::vector<long long>>(args[0])));
}
