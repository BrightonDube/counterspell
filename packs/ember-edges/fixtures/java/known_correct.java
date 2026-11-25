// Test-only known-correct Java fixture for ember-edges.
public class Solution {
    private static long lower(long[] embers, long mark) {
        long lo = 0, hi = embers.length;
        while (lo < hi) {
            long mid = (lo + hi) / 2;
            if (embers[(int) mid] < mark) { lo = mid + 1; } else { hi = mid; }
        }
        return lo;
    }
    public static long[] ember_edges(long[] embers, long target) {
        long first = lower(embers, target);
        if (first == embers.length || embers[(int) first] != target) {
            return new long[]{-1, -1};
        }
        return new long[]{first, lower(embers, target + 1) - 1};
    }
}
