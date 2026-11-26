// Test-only known-correct C++ fixture for ember-partition.
static long long go(const std::vector<long long>& marks, size_t i, long long target,
                    std::vector<long long>& sums, std::vector<int>& counts) {
    if (i == marks.size()) {
        for (int c : counts) {
            if (c == 0) {
                return 0;
            }
        }
        return 1;
    }
    long long placed = 0;
    for (size_t g = 0; g < sums.size(); g += 1) {
        if (counts[g] == 0 && g > 0 && counts[g - 1] == 0) {
            break;
        }
        if (sums[g] + marks[i] > target) {
            continue;
        }
        sums[g] += marks[i];
        counts[g] += 1;
        placed += go(marks, i + 1, target, sums, counts);
        sums[g] -= marks[i];
        counts[g] -= 1;
        if (counts[g] == 0) {
            break;
        }
    }
    return placed;
}

long long partition_count(std::vector<long long> marks, long long ways) {
    long long total = 0;
    for (long long m : marks) {
        total += m;
    }
    if (total % ways != 0 || marks.size() < (size_t) ways) {
        return 0;
    }
    std::vector<long long> sums(ways, 0);
    std::vector<int> counts(ways, 0);
    return go(marks, 0, total / ways, sums, counts);
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(partition_count(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));
}
