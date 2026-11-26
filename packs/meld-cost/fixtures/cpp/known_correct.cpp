// Test-only known-correct C++ fixture for meld-cost.
long long meld_cost(std::vector<long long> sizes) {
    std::priority_queue<long long, std::vector<long long>, std::greater<long long>> heap(
        sizes.begin(), sizes.end());
    long long cost = 0;
    while (heap.size() > 1) {
        long long a = heap.top(); heap.pop();
        long long b = heap.top(); heap.pop();
        cost += a + b;
        heap.push(a + b);
    }
    return cost;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(meld_cost(from_json<std::vector<long long>>(args[0])));
}
