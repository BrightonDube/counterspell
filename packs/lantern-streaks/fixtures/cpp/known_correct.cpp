// Test-only known-correct C++ fixture for lantern-streaks (language-drivers T15).
// Convention: `counterspell_call` converts each argument with `from_json`,
// calls the typed function, and wraps the return value with `to_json`.
long long lit_streak(std::vector<long long> lights) {
    long long streak = 0;
    for (std::size_t index = lights.size(); index > 0; index -= 1) {
        if (lights[index - 1] == 0) break;
        streak += 1;
    }
    return streak;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(lit_streak(from_json<std::vector<long long>>(args[0])));
}
