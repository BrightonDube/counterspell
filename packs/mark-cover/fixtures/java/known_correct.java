// Test-only known-correct Java fixture for mark-cover.
public class Solution {
    public static long cover_length(long[][] spans) {
        boolean[] cov = new boolean[101];
        for (long[] s : spans) {
            for (long i = s[0]; i <= s[1]; i += 1) {
                cov[(int) i] = true;
            }
        }
        long total = 0;
        for (boolean c : cov) {
            if (c) {
                total += 1;
            }
        }
        return total;
    }
}
