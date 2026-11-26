// Test-only known-correct Java fixture for lit-combos.
public class Solution {
    public static long count_mixes(long[] coins, long target) {
        int t = (int) target;
        long[] d = new long[t + 1];
        d[0] = 1;
        for (long c : coins) {
            for (int i = (int) c; i <= t; i += 1) {
                d[i] += d[(int) (i - c)];
            }
        }
        return d[t];
    }
}
