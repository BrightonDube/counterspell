// Test-only known-correct Java fixture for median-ward.
public class Solution {
    public static long median_x2(long[] marks) {
        long[] m = marks.clone();
        java.util.Arrays.sort(m);
        int n = m.length;
        if (n == 0) {
            return 0;
        }
        if (n % 2 == 1) {
            return 2 * m[n / 2];
        }
        return m[n / 2 - 1] + m[n / 2];
    }
}
