// Test-only known-correct Java fixture for chunk-toll.
public class Solution {
    public static long chunk_toll(long[] tokens, long w, long base) {
        int n = tokens.length;
        int width = (int) Math.max(w, 1);
        long[] dp = new long[n + 1];
        java.util.Arrays.fill(dp, Long.MAX_VALUE / 4);
        dp[0] = 0;
        for (int i = 1; i <= n; i++) {
            for (int j = Math.max(0, i - width); j < i; j++) {
                long hi = Long.MIN_VALUE, lo = Long.MAX_VALUE;
                for (int k = j; k < i; k++) { hi = Math.max(hi, tokens[k]); lo = Math.min(lo, tokens[k]); }
                long cost = dp[j] + base + (hi - lo);
                if (cost < dp[i]) dp[i] = cost;
            }
        }
        return dp[n];
    }
}
