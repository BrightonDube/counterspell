// Test-only known-correct Java fixture for join-span.
public class Solution {
    public static long join_span(long[][] a, long[][] b) {
        long count = 0;
        for (long[] x : a) {
            for (long[] y : b) {
                if (x[0] == y[0] && x[1] < y[1]) {
                    count += 1;
                }
            }
        }
        return count;
    }
}
