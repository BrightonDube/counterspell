// Test-only known-correct C++ fixture for summon-rite.
static bool go(const std::vector<long long>& marks, size_t i, long long need) {
    if (need == 0) {
        return true;
    }
    if (i >= marks.size()) {
        return false;
    }
    return go(marks, i + 1, need - marks[i]) || go(marks, i + 1, need);
}

long long can_summon(std::vector<long long> marks, long long target) {
    return go(marks, 0, target) ? 1 : 0;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(can_summon(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
