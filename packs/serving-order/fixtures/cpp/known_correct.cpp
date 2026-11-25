// Test-only known-correct C++ fixture for serving-order.
std::vector<long long> serve_order(std::vector<long long> ops) {
    std::vector<long long> line(ops.size());
    std::vector<long long> served;
    std::size_t front = 0;
    std::size_t back = 0;
    for (long long op : ops) {
        if (op == 0) {
            if (front < back) {
                served.push_back(line[front]);
                front += 1;
            }
        } else {
            line[back] = op;
            back += 1;
        }
    }
    return served;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(serve_order(from_json<std::vector<long long>>(args[0])));
}
