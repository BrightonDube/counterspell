// Test-only known-correct Java fixture for idle-stretch.
public class Solution {
    public static long idle_stretch(long[][] spans, long t) {
        int n = (int) t + 1;
        boolean[] cov = new boolean[n];
        for (long[] s : spans) {
            for (long i = Math.max(s[0], 0); i <= Math.min(s[1], t); i += 1) {
                cov[(int) i] = true;
            }
        }
        long best = 0;
        long run = 0;
        for (int i = 0; i < n; i += 1) {
            run = cov[i] ? 0 : run + 1;
            if (run > best) {
                best = run;
            }
        }
        return best;
    }
}
