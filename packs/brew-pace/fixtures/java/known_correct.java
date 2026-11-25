// Test-only known-correct Java fixture for brew-pace.
public class Solution {
    public static long min_pace(long[] brews, long hours) {
        long lo = 1, hi = 1;
        for (long t : brews) { if (t > hi) { hi = t; } }
        while (lo < hi) {
            long mid = (lo + hi) / 2;
            long need = 0;
            for (long t : brews) { need += (t + mid - 1) / mid; }
            if (need <= hours) { hi = mid; } else { lo = mid + 1; }
        }
        return lo;
    }
}
