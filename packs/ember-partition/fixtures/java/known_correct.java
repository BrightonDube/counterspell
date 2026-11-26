// Test-only known-correct Java fixture for ember-partition.
public class Solution {
    static long go(long[] marks, int i, long target, long[] sums, int[] counts) {
        if (i == marks.length) {
            for (int c : counts) {
                if (c == 0) {
                    return 0;
                }
            }
            return 1;
        }
        long placed = 0;
        for (int g = 0; g < sums.length; g += 1) {
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
    public static long partition_count(long[] marks, long ways) {
        long total = 0;
        for (long m : marks) {
            total += m;
        }
        int w = (int) ways;
        if (total % ways != 0 || marks.length < w) {
            return 0;
        }
        return go(marks, 0, total / ways, new long[w], new int[w]);
    }
}
