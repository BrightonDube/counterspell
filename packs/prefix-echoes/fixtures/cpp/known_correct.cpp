// Test-only known-correct C++ fixture for prefix-echoes (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
std::vector<long long> prefix_counts(std::vector<long long> marks,
                                     std::vector<std::vector<long long>> queries) {
    std::vector<long long> answers;
    answers.reserve(queries.size());
    for (const auto& query : queries) {
        long long count = 0;
        std::size_t k = std::min<std::size_t>(query[0], marks.size());
        for (std::size_t i = 0; i < k; i += 1) {
            if (marks[i] == query[1]) count += 1;
        }
        answers.push_back(count);
    }
    return answers;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(prefix_counts(from_json<std::vector<long long>>(args[0]),
                                 from_json<std::vector<std::vector<long long>>>(args[1])));
}
