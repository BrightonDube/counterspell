// Test-only known-correct Java fixture for ember-split.
public class Solution {
    static long go(long[] marks, int i, long need) {
        if (need == 0) {
            return 1;
        }
        if (i == marks.length) {
            return 0;
        }
        return go(marks, i + 1, need - marks[i]) + go(marks, i + 1, need);
    }
    public static long split_count(long[] marks) {
        long total = 0;
        for (long m : marks) {
            total += m;
        }
        if (total % 2 != 0) {
            return 0;
        }
        long half = total / 2;
        if (half == 0) {
            return ((1L << marks.length) - 2) / 2;
        }
        return go(marks, 0, half) / 2;
    }
}
