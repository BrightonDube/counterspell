// Test-only known-correct Java fixture for valley-low.
public class Solution {
    public static long lowest_mark(long[] rotated) {
        long lo = 0, hi = rotated.length - 1;
        while (lo < hi) {
            long mid = (lo + hi) / 2;
            if (rotated[(int) mid] > rotated[(int) hi]) { lo = mid + 1; } else { hi = mid; }
        }
        return rotated[(int) lo];
    }
}
