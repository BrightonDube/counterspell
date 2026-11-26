// Test-only known-correct Java fixture for ward-rows.
public class Solution {
    public static long row_ways(long length) {
        if (length <= 0) {
            return 1;
        }
        int n = (int) length;
        long[] d = new long[n + 1];
        d[0] = 1;
        for (int i = 1; i <= n; i += 1) {
            for (int p = 1; p <= 3; p += 1) {
                if (i - p >= 0) {
                    d[i] += d[i - p];
                }
            }
        }
        return d[n];
    }
}
