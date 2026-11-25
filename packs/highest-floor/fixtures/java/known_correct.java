// Test-only known-correct Java fixture for highest-floor.
public class Solution {
    public static long highest_floor(long[] steps, long reach) {
        long lo = 0, hi = steps.length;
        while (lo < hi) {
            long mid = (lo + hi) / 2;
            if (steps[(int) mid] <= reach) { lo = mid + 1; } else { hi = mid; }
        }
        return lo > 0 ? steps[(int) lo - 1] : -1;
    }
}
