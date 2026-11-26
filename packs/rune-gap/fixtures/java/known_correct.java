// Test-only known-correct Java fixture for rune-gap.
public class Solution {
    static long go(long[] marks, int i, long left) {
        if (left < 0) {
            return 0;
        }
        if (i == marks.length) {
            return 1;
        }
        return go(marks, i + 1, left - marks[i]) + go(marks, i + 1, left);
    }
    public static long gap_count(long[] marks, long cap) {
        return go(marks, 0, cap);
    }
}
