// Test-only known-correct C++ fixture for wall-span.
long long widest_fit(std::vector<long long> wall) {
    long long best = 0;
    std::vector<long long> stack;
    for (long long index = 0; index <= (long long)wall.size(); index += 1) {
        long long height = index < (long long)wall.size() ? wall[index] : 0;
        while (!stack.empty() && height < wall[stack.back()]) {
            long long popped = stack.back();
            stack.pop_back();
            long long left = stack.empty() ? 0 : stack.back() + 1;
            long long area = wall[popped] * (index - left);
            if (area > best) {
                best = area;
            }
        }
        stack.push_back(index);
    }
    return best;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(widest_fit(from_json<std::vector<long long>>(args[0])));
}
