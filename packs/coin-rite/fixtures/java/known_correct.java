// Test-only known-correct Java fixture for coin-rite.
public class Solution {
    public static long coin_rite(long[] coins, long target) {
        int t = (int) target;
        int[] d = new int[t + 1];
        java.util.Arrays.fill(d, Integer.MAX_VALUE / 2);
        d[0] = 0;
        for (int i = 1; i <= t; i += 1) {
            for (long c : coins) {
                if (i - c >= 0) {
                    d[i] = Math.min(d[i], d[(int) (i - c)] + 1);
                }
            }
        }
        return d[t] >= Integer.MAX_VALUE / 2 ? -1 : d[t];
    }
}
