// Test-only known-correct Java fixture for row-join.
public class Solution {
    public static long row_join(long[][] a, long[][] b) {
        long count = 0;
        for (long[] x : a) {
            for (long[] y : b) {
                if (x[0] == y[0]) {
                    count += 1;
                }
            }
        }
        return count;
    }
}
