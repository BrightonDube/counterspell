// Test-only known-correct Java fixture for null-join.
public class Solution {
    public static long null_join(long[][] a, long[][] b) {
        long count = 0;
        for (long[] x : a) {
            for (long[] y : b) {
                if (x[0] == y[0] && x[0] != -1) {
                    count += 1;
                }
            }
        }
        return count;
    }
}
