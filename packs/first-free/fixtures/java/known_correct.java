// Test-only known-correct Java fixture for first-free.
public class Solution {
    public static long first_free(long[][] spans) {
        boolean[] cov = new boolean[102];
        for (long[] s : spans) {
            for (long i = s[0]; i <= s[1]; i += 1) {
                cov[(int) i] = true;
            }
        }
        long i = 0;
        while (i < 101 && cov[(int) i]) {
            i += 1;
        }
        return i;
    }
}
