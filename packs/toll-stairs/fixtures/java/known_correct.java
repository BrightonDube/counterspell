// Test-only known-correct Java fixture for toll-stairs.
public class Solution {
    public static long toll_stairs(long[] costs) {
        int n = costs.length;
        if (n <= 1) {
            return n == 1 ? costs[0] : 0;
        }
        long[] d = new long[n + 1];
        for (int i = 2; i <= n; i += 1) {
            d[i] = Math.min(d[i - 1] + costs[i - 1], d[i - 2] + costs[i - 2]);
        }
        return d[n];
    }
}
