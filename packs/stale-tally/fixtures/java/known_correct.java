// Test-only known-correct Java fixture for stale-tally.
public class Solution {
    public static long stale_count(long[] seen, long[] queued) {
        long n = 0;
        for (int i = 0; i < seen.length; i += 1) {
            if (seen[i] != queued[i]) {
                n += 1;
            }
        }
        return n;
    }
}
