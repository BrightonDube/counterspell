// Test-only known-correct Java fixture for count-below.
public class Solution {
    public static long count_below(long[] marks, long toll) {
        long lo = 0, hi = marks.length;
        while (lo < hi) {
            long mid = (lo + hi) / 2;
            if (marks[(int) mid] < toll) { lo = mid + 1; } else { hi = mid; }
        }
        return lo;
    }
}
