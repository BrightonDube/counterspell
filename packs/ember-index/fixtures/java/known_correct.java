// Test-only known-correct Java fixture for ember-index.
public class Solution {
    public static long find_ember(long[] embers, long target) {
        long lo = 0, hi = embers.length;
        while (lo < hi) {
            long mid = (lo + hi) / 2;
            if (embers[(int) mid] < target) { lo = mid + 1; } else { hi = mid; }
        }
        return lo < embers.length && embers[(int) lo] == target ? lo : -1;
    }
}
