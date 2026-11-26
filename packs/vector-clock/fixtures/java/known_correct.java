// Test-only known-correct Java fixture for vector-clock.
public class Solution {
    public static long clock_cmp(long[] a, long[] b) {
        boolean less = false, more = false;
        for (int i = 0; i < a.length; i++) {
            if (a[i] < b[i]) less = true;
            if (a[i] > b[i]) more = true;
        }
        if (less && more) return 2;
        if (less) return -1;
        if (more) return 1;
        return 0;
    }
}
