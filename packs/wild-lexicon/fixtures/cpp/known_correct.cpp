// Test-only known-correct C++ fixture for wild-lexicon.
static bool match_word(const std::string& w, const std::string& p) {
    size_t n = w.size();
    size_t m = p.size();
    std::vector<std::vector<bool>> dp(n + 1, std::vector<bool>(m + 1, false));
    dp[0][0] = true;
    for (size_t j = 1; j <= m; j += 1) {
        if (p[j - 1] == '*') {
            dp[0][j] = dp[0][j - 1];
        }
    }
    for (size_t i = 1; i <= n; i += 1) {
        for (size_t j = 1; j <= m; j += 1) {
            dp[i][j] =
                (p[j - 1] == '*' && (dp[i][j - 1] || dp[i - 1][j]))
                || (p[j - 1] == w[i - 1] && dp[i - 1][j - 1]);
        }
    }
    return dp[n][m];
}

long long count_matches(std::string words, std::string pattern) {
    std::set<std::string> ws;
    std::string cur;
    std::stringstream stream(words);
    while (std::getline(stream, cur, ',')) {
        if (!cur.empty()) {
            ws.insert(cur);
        }
    }
    long long ans = 0;
    for (const auto& w : ws) {
        if (match_word(w, pattern)) {
            ans += 1;
        }
    }
    return ans;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(count_matches(from_json<std::string>(args[0]), from_json<std::string>(args[1])));
}
