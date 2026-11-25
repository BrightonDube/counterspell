// Test-only known-correct Java fixture for first-clear.
public class Solution {
    public static long first_clear(long[] heights, long mark) {
        long lo = 0, hi = heights.length;
        while (lo < hi) {
            long mid = (lo + hi) / 2;
            if (heights[(int) mid] < mark) { lo = mid + 1; } else { hi = mid; }
        }
        return lo;
    }
}
